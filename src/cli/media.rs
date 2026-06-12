//! `vmlab media build` — folder → ISO/floppy image (PRD §6.3).

use std::path::PathBuf;

use anyhow::Result;

#[derive(clap::Subcommand)]
pub enum MediaCmd {
    /// Build an image from a folder
    Build {
        /// "iso" or "floppy"
        kind: String,
        /// Source folder
        from: PathBuf,
        /// Output image path
        out: PathBuf,
        /// Volume label
        #[arg(short, long)]
        label: Option<String>,
    },
}

pub fn cmd_media(cmd: MediaCmd) -> Result<()> {
    match cmd {
        MediaCmd::Build {
            kind,
            from,
            out,
            label,
        } => match kind.as_str() {
            "iso" => {
                crate::media::build_iso(&from, &out, label.as_deref())?;
                println!("built ISO {}", out.display());
                Ok(())
            }
            "floppy" => {
                crate::media::build_floppy(&from, &out, label.as_deref())?;
                println!("built floppy {}", out.display());
                Ok(())
            }
            other => anyhow::bail!("unknown media kind `{other}` (expected iso, floppy)"),
        },
    }
}
