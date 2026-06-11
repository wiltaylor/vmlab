//! Template store and supporting machinery (PRD §4, §6, §7.1).
//!
//! - [`meta`] — `template.wcl` metadata read/written beside each disk image.
//! - [`qimg`] — async `qemu-img` wrappers (blank disks, linked clones, info).
//! - [`store`] — the on-disk store at `~/.local/share/vmlab/templates`.

// Buildout in progress: consumers of these re-exports land later (the
// crate root's dead_code allow does not cover unused imports).
#![allow(unused_imports)]

pub mod meta;
pub mod qimg;
pub mod store;

pub use meta::{META_FILE, TemplateMeta};
pub use qimg::{ImageInfo, QemuImgError, create_blank, create_linked_clone, image_info, resize};
pub use store::{DISK_FILE, ResolvedTemplate, TemplateStore, compare_versions, sha256_file};
