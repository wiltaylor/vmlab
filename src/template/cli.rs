//! `vmlab template ...` — store management, builds, OCI distribution (PRD
//! §6, §12). Runs in the CLI process (template store writes are serialised
//! by the store's own file lock).

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result, anyhow, bail};

use super::build::build_template;
use super::store::TemplateStore;
use crate::config::model::parse_template_ref;

#[derive(clap::Subcommand)]
pub enum TemplateCmd {
    /// Build templates defined in a lab/template file
    Build {
        /// File containing `template {}` blocks (default: ./vmlab.wcl)
        #[arg(short, long)]
        file: Option<PathBuf>,
        /// Build only the named template (default: all in the file)
        name: Option<String>,
    },
    /// List templates in the store
    List {
        /// Emit a JSON array instead of a table
        #[arg(long)]
        json: bool,
    },
    /// Check whether a template is in the store (`<arch>/<name>[@<version>]`).
    /// Prints the resolved ref and exits 0 if present; exits nonzero if not.
    Exists { reference: String },
    /// Remove a template (`<arch>/<name>[@<version>]`)
    Rm {
        reference: String,
        /// Remove even if it backs existing clones
        #[arg(long)]
        force: bool,
    },
    /// Export a template to a portable archive
    Export {
        reference: String,
        /// Output archive path (.tar.zst)
        out: PathBuf,
    },
    /// Import a template from an archive
    Import {
        archive: PathBuf,
        /// Overwrite an existing version
        #[arg(long)]
        overwrite: bool,
    },
    /// Push a template to an OCI registry
    Push {
        /// Local template `<arch>/<name>[@<version>]`
        reference: String,
        /// Registry reference, e.g. ghcr.io/owner/name:version
        target: String,
    },
    /// Pull a template from an OCI registry
    Pull {
        /// Registry reference, e.g. ghcr.io/owner/name:version
        target: String,
        /// Architecture to pull (required for multi-arch indexes)
        #[arg(long)]
        arch: Option<String>,
        /// Overwrite an existing version in the store
        #[arg(long)]
        overwrite: bool,
    },
    /// Log in to an OCI registry
    Login {
        registry: String,
        #[arg(short, long)]
        username: String,
        #[arg(short, long)]
        password: String,
    },
}

fn store() -> TemplateStore {
    TemplateStore::new(crate::paths::template_store_dir())
}

pub fn cmd_template(cmd: TemplateCmd) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        match cmd {
            TemplateCmd::Build { file, name } => build(file, name).await,
            TemplateCmd::List { json } => list(json),
            TemplateCmd::Exists { reference } => exists(&reference),
            TemplateCmd::Rm { reference, force } => rm(&reference, force),
            TemplateCmd::Export { reference, out } => export(&reference, &out),
            TemplateCmd::Import { archive, overwrite } => import(&archive, overwrite),
            TemplateCmd::Push { reference, target } => push(&reference, &target).await,
            TemplateCmd::Pull {
                target,
                arch,
                overwrite,
            } => pull(&target, arch.as_deref(), overwrite).await,
            TemplateCmd::Login {
                registry,
                username,
                password,
            } => login(&registry, &username, &password).await,
        }
    })
}

async fn build(file: Option<PathBuf>, only: Option<String>) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let path = match file {
        Some(p) => p,
        None => crate::paths::find_lab_root(&cwd)?.join(crate::paths::LAB_FILE),
    };
    let root = path.parent().unwrap_or(&cwd).to_path_buf();
    let source =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let tf = crate::config::load_template_source(&source, &path.display().to_string(), &root)
        .map_err(|e| anyhow!("{:?}", miette::Report::new(e)))?;
    if tf.templates.is_empty() {
        bail!("no `template {{}}` blocks in {}", path.display());
    }
    let profiles = crate::profiles::ProfileSet::load_default()?;
    let store = store();
    let log: crate::scripting::OutputSink = Arc::new(|line: String| print!("{line}"));

    let targets: Vec<_> = tf
        .templates
        .iter()
        .filter(|t| only.as_deref().is_none_or(|n| n == t.name))
        .collect();
    if targets.is_empty() {
        bail!(
            "no template named \"{}\" in {}",
            only.unwrap_or_default(),
            path.display()
        );
    }
    for def in targets {
        build_template(def, &root, &store, &profiles, log.clone())
            .await
            .with_context(|| format!("building {}/{}@{}", def.arch, def.name, def.version))?;
    }
    Ok(())
}

fn list(json: bool) -> Result<()> {
    let templates = store().list()?;
    if json {
        let entries: Vec<_> = templates.iter().map(meta_json).collect();
        println!("{}", serde_json::to_string_pretty(&entries)?);
        return Ok(());
    }
    if templates.is_empty() {
        println!("no templates in the store");
        return Ok(());
    }
    println!("{:<10} {:<28} {:<12} CREATED", "ARCH", "NAME", "VERSION");
    for t in templates {
        println!(
            "{:<10} {:<28} {:<12} {}",
            t.arch,
            t.name,
            t.version,
            t.created.format("%Y-%m-%d")
        );
    }
    Ok(())
}

/// Fixed-shape JSON for scripting: every key always present (null when
/// the template does not record it), sizes in bytes, created as RFC 3339.
fn meta_json(t: &crate::template::meta::TemplateMeta) -> serde_json::Value {
    serde_json::json!({
        "arch": t.arch,
        "name": t.name,
        "version": t.version,
        "ref": format!("{}/{}@{}", t.arch, t.name, t.version),
        "profile": t.profile,
        "cpus": t.cpus,
        "memory": t.memory,
        "disk": t.disk,
        "firmware": t.firmware,
        "tpm": t.tpm,
        "secure_boot": t.secure_boot,
        "display": t.display,
        "created": t.created.to_rfc3339(),
        "origin": t.origin,
        "sha256": t.sha256,
    })
}

fn exists(reference: &str) -> Result<()> {
    let (arch, name, version) = parse_store_ref(reference)?;
    let resolved =
        store()
            .resolve(&arch, &name, version.as_deref())
            .map_err(|_| match &version {
                Some(v) => anyhow!("{arch}/{name}@{v} not in the store"),
                None => anyhow!("{arch}/{name} not in the store"),
            })?;
    println!(
        "{}/{}@{}",
        resolved.meta.arch, resolved.meta.name, resolved.meta.version
    );
    Ok(())
}

fn rm(reference: &str, force: bool) -> Result<()> {
    let (arch, name, version) = parse_store_ref(reference)?;
    let version = version.ok_or_else(|| {
        anyhow!("specify the exact version to remove, e.g. {arch}/{name}@<version>")
    })?;
    store().remove(&arch, &name, &version, force, &|_| {
        if force {
            None
        } else {
            Some(
                "deleting a template may break existing linked clones; re-run with --force"
                    .to_string(),
            )
        }
    })?;
    println!("removed {arch}/{name}@{version}");
    Ok(())
}

fn export(reference: &str, out: &std::path::Path) -> Result<()> {
    let (arch, name, version) = parse_store_ref(reference)?;
    store().export(&arch, &name, version.as_deref(), out)?;
    println!("exported to {}", out.display());
    Ok(())
}

fn import(archive: &std::path::Path, overwrite: bool) -> Result<()> {
    let meta = store().import(archive, overwrite)?;
    println!("imported {}/{}@{}", meta.arch, meta.name, meta.version);
    Ok(())
}

async fn push(reference: &str, target: &str) -> Result<()> {
    let (arch, name, version) = parse_store_ref(reference)?;
    let resolved = store().resolve(&arch, &name, version.as_deref())?;
    let target = crate::oci::version_in_repo_path(target, Some(&resolved.meta.version))?;
    let host_cfg = crate::config::host::HostConfig::load_default().unwrap_or_default();
    super::oci_bridge::push(&resolved.dir, &target, host_cfg.oci_chunk_size, &arch)
        .await
        .context("pushing to registry")?;
    println!("pushed {arch}/{name}@{} to {target}", resolved.meta.version);
    Ok(())
}

async fn pull(target: &str, arch: Option<&str>, overwrite: bool) -> Result<()> {
    let target = crate::oci::version_in_repo_path(target, None)?;
    let store = store();
    let meta = super::oci_bridge::pull(&target, arch, &store, overwrite)
        .await
        .context("pulling from registry")?;
    println!(
        "pulled {}/{}@{} into the store",
        meta.arch, meta.name, meta.version
    );
    Ok(())
}

async fn login(registry: &str, username: &str, password: &str) -> Result<()> {
    super::oci_bridge::login(registry, username, password).await?;
    println!("logged in to {registry}");
    Ok(())
}

fn parse_store_ref(reference: &str) -> Result<(String, String, Option<String>)> {
    match parse_template_ref(reference).map_err(|e| anyhow!(e))? {
        crate::config::model::TemplateRef::Store {
            arch,
            name,
            version,
        } => Ok((arch, name, version)),
        _ => bail!("expected a local store reference `<arch>/<name>[@<version>]`"),
    }
}
