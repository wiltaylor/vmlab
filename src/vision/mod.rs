//! Vision support for VM automation (PRD §10.3 "Screen").
//!
//! Provides the host-side primitives behind `vm.wait_for_image`,
//! `vm.find_image`, `vm.ocr` and `vm.wait_for_text`:
//!
//! - [`load_screen`] / [`save_png`] — read QMP `screendump` output (PPM) or
//!   PNG reference images, write PNG screenshots.
//! - [`find_template`] — normalised cross-correlation template matching,
//!   returning a [`Match`] (location + score) that can anchor a relative
//!   mouse click.
//! - [`ocr`] / [`text_matches`] — Tesseract-backed text extraction and regex
//!   matching.
//!
//! Wait/retry loops and lab-relative path resolution live in the scripting
//! layer; this module is pure image-in, result-out.

mod matching;
mod ocr;
mod screenshot;

// The consumer (wisp scripting bridge) lands later in the buildout; until
// then the re-exports are intentionally unused.
#[allow(unused_imports)]
pub use matching::{Match, MatchOptions, find_template};
#[allow(unused_imports)]
pub use ocr::{ocr, text_matches};
#[allow(unused_imports)]
pub use screenshot::{load_screen, save_png};
