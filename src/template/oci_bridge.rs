//! Thin bridge to the OCI distribution module (PRD §6.4). Kept as a seam so
//! the template CLI compiles independently of the OCI module's exact API;
//! reconciled to the real `crate::oci` calls once that module lands.

use std::path::Path;

use anyhow::{Result, bail};

use super::meta::TemplateMeta;
use super::store::TemplateStore;

pub async fn push(
    _template_dir: &Path,
    _target: &str,
    _chunk_size: u64,
    _arch: &str,
) -> Result<()> {
    bail!("OCI push is not yet wired into this build")
}

pub async fn pull(
    _target: &str,
    _arch: Option<&str>,
    _store: &TemplateStore,
) -> Result<TemplateMeta> {
    bail!("OCI pull is not yet wired into this build")
}

pub async fn login(_registry: &str, _user: &str, _pass: &str) -> Result<()> {
    bail!("OCI login is not yet wired into this build")
}
