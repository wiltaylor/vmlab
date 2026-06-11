//! QEMU integration: hardware resolution, command-line construction,
//! firmware lookup, process management (PRD §3, §5.2).

pub mod cmdline;
pub mod firmware;
pub mod process;
pub mod resolve;

#[allow(unused_imports)]
pub use cmdline::{Accel, VmPaths, build_args, emulator_binary, pick_accel};
#[allow(unused_imports)]
pub use process::Proc;
#[allow(unused_imports)]
pub use resolve::{ResolvedVm, resolve_vm};
