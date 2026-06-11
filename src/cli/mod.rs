//! CLI surface (PRD §12). The same binary also hosts the supervisor and lab
//! daemons via hidden subcommands, re-exec'd from the CLI as needed.

pub mod daemon;
mod validate;

use clap::{Parser, Subcommand};
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "vmlab", version, about = "Single-host VM lab orchestrator")]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Validate the lab file with no side effects
    Validate,
    /// Supervisor control (normally automatic)
    Daemon {
        #[command(subcommand)]
        cmd: daemon::DaemonCmd,
    },
    /// Internal: run the supervisor daemon in the foreground
    #[command(name = "__supervisord", hide = true)]
    Supervisord,
    /// Internal: run a lab daemon in the foreground
    #[command(name = "__labd", hide = true)]
    Labd {
        /// Lab name
        #[arg(long)]
        lab: String,
        /// Directory containing vmlab.wcl
        #[arg(long)]
        root: std::path::PathBuf,
    },
}

pub fn run() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Validate => validate::cmd_validate(),
        Command::Daemon { cmd } => daemon::cmd_daemon(cmd),
        Command::Supervisord => {
            init_daemon_tracing();
            crate::supervisor::run()
        }
        Command::Labd { lab, root } => {
            init_daemon_tracing();
            crate::labd::run(lab, root)
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            // ConfigErrors render as rich miette reports; everything else as
            // a plain error chain.
            eprintln!("{err:?}");
            ExitCode::FAILURE
        }
    }
}

fn init_daemon_tracing() {
    use tracing_subscriber::EnvFilter;
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with_target(false)
        .init();
}
