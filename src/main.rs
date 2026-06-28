//! The `vmlab` CLI binary. All logic lives in the `vmlab` library crate; this
//! binary is a thin entrypoint. The same binary also hosts the supervisor and
//! lab daemons via hidden subcommands (see `vmlab::cli`).

fn main() -> std::process::ExitCode {
    vmlab::cli::run()
}
