// Buildout in progress: items land before their consumers. Remove once the
// CLI surface is complete (PRD §12).
#![allow(dead_code)]

mod cli;
mod config;
mod labd;
mod net;
mod paths;
mod profiles;
mod proto;
mod qemu;
mod qga;
mod qmp;
mod supervisor;
mod template;

fn main() -> std::process::ExitCode {
    cli::run()
}
