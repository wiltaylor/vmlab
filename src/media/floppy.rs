//! Floppy image building from folders (PRD §6.3).
//!
//! Creates a blank 1.44 MB image, FAT12-formats it with `mformat`, and
//! copies the folder contents in with `mcopy -s`. mcopy handles VFAT long
//! names itself; failures surface mtools stderr verbatim.

use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail, ensure};

/// Raw size of a 1.44 MB floppy image: 2880 sectors of 512 bytes.
const FLOPPY_IMAGE_BYTES: u64 = 2880 * SECTOR;
/// FAT12 sector / cluster size on a 1.44 MB floppy.
const SECTOR: u64 = 512;
/// Usable data area: 2880 sectors minus 1 boot sector, two 9-sector FATs,
/// and the 14-sector root directory.
const FLOPPY_DATA_BYTES: u64 = (2880 - 1 - 2 * 9 - 14) * SECTOR;
/// Maximum FAT volume label length.
const FLOPPY_LABEL_MAX: usize = 11;

/// Builds a 1.44 MB FAT12 floppy image at `out` from the contents of
/// `src_folder`.
pub fn build_floppy(src_folder: &Path, out: &Path, label: Option<&str>) -> Result<()> {
    ensure!(
        src_folder.is_dir(),
        "floppy source {} is not a directory",
        src_folder.display()
    );
    let label = label.map(validate_floppy_label).transpose()?;
    check_capacity(src_folder)?;

    // Blank image first; mformat formats in place.
    fs::write(out, vec![0u8; FLOPPY_IMAGE_BYTES as usize])
        .with_context(|| format!("creating blank floppy image {}", out.display()))?;

    let mut mformat = Command::new("mformat");
    mformat.arg("-i").arg(out).args(["-f", "1440"]);
    if let Some(label) = &label {
        mformat.arg("-v").arg(label);
    }
    mformat.arg("::");
    run_mtool(mformat, "mformat")?;

    let mut entries: Vec<_> = fs::read_dir(src_folder)
        .with_context(|| format!("reading floppy source {}", src_folder.display()))?
        .map(|item| item.map(|item| item.path()))
        .collect::<Result<_, _>>()
        .with_context(|| format!("reading floppy source {}", src_folder.display()))?;
    entries.sort();
    if entries.is_empty() {
        return Ok(());
    }

    let mut mcopy = Command::new("mcopy");
    mcopy.arg("-i").arg(out).arg("-s");
    mcopy.args(&entries);
    mcopy.arg("::/");
    run_mtool(mcopy, "mcopy")?;
    Ok(())
}

/// Runs an mtools command, surfacing stderr on failure.
fn run_mtool(mut cmd: Command, name: &str) -> Result<()> {
    let output = cmd
        .output()
        .with_context(|| format!("running {name} (are mtools installed?)"))?;
    if !output.status.success() {
        bail!(
            "{name} failed ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

/// Errors if the folder contents cannot fit the floppy's FAT12 data area,
/// reporting the overage. Sizes are rounded up to whole 512-byte clusters
/// and each subdirectory is charged one cluster for its entries.
fn check_capacity(src_folder: &Path) -> Result<()> {
    let used = dir_cluster_bytes(src_folder)
        .with_context(|| format!("sizing floppy source {}", src_folder.display()))?;
    ensure!(
        used <= FLOPPY_DATA_BYTES,
        "contents of {} need {used} bytes after FAT12 cluster rounding, but a 1.44MB floppy \
         holds {FLOPPY_DATA_BYTES} bytes: {} bytes over capacity",
        src_folder.display(),
        used - FLOPPY_DATA_BYTES
    );
    Ok(())
}

/// Cluster-rounded bytes consumed by the contents of `dir` (not counting
/// `dir`'s own directory entry, which for the source root lives in the
/// reserved root-directory sectors).
fn dir_cluster_bytes(dir: &Path) -> Result<u64> {
    let mut total = 0u64;
    for item in fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        let item = item.with_context(|| format!("reading {}", dir.display()))?;
        let path = item.path();
        if path.is_dir() {
            // A subdirectory occupies at least one cluster for its entries.
            total += SECTOR + dir_cluster_bytes(&path)?;
        } else {
            let len = fs::metadata(&path)
                .with_context(|| format!("reading metadata of {}", path.display()))?
                .len();
            total += len.div_ceil(SECTOR) * SECTOR;
        }
    }
    Ok(total)
}

/// Validates a FAT volume label: at most 11 characters, uppercased, from a
/// conservative character set. Returns the uppercased label.
fn validate_floppy_label(label: &str) -> Result<String> {
    ensure!(!label.is_empty(), "floppy label must not be empty");
    ensure!(
        label.len() <= FLOPPY_LABEL_MAX,
        "floppy label {label:?} is {} characters; the maximum is {FLOPPY_LABEL_MAX}",
        label.len()
    );
    let upper = label.to_ascii_uppercase();
    ensure!(
        upper.bytes().all(|b| b.is_ascii_uppercase()
            || b.is_ascii_digit()
            || matches!(b, b' ' | b'_' | b'-')),
        "floppy label {label:?} contains invalid characters: use A-Z, 0-9, space, '_', '-'"
    );
    Ok(upper)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::process::Command;

    use super::*;

    /// Recursive listing of a floppy image via `mdir -i <img> -/ ::`.
    fn floppy_listing(img: &Path) -> String {
        let output = Command::new("mdir")
            .arg("-i")
            .arg(img)
            .args(["-/", "::"])
            .output()
            .expect("mdir should be runnable");
        assert!(output.status.success(), "mdir failed: {output:?}");
        String::from_utf8_lossy(&output.stdout).into_owned()
    }

    #[test]
    fn builds_floppy_with_nested_files() {
        let src = tempfile::tempdir().unwrap();
        fs::write(src.path().join("hello.txt"), b"hello floppy").unwrap();
        fs::create_dir_all(src.path().join("payload")).unwrap();
        fs::write(src.path().join("payload/nested.cfg"), b"key=value").unwrap();

        let out_dir = tempfile::tempdir().unwrap();
        let img = out_dir.path().join("media.img");
        build_floppy(src.path(), &img, Some("BOOTME")).expect("floppy build should succeed");

        assert_eq!(fs::metadata(&img).unwrap().len(), FLOPPY_IMAGE_BYTES);
        let listing = floppy_listing(&img).to_ascii_lowercase();
        assert!(listing.contains("hello"), "listing: {listing}");
        assert!(listing.contains("payload"), "listing: {listing}");
        assert!(listing.contains("nested"), "listing: {listing}");
    }

    #[test]
    fn rejects_contents_over_capacity() {
        let src = tempfile::tempdir().unwrap();
        fs::write(
            src.path().join("big.bin"),
            vec![0xAAu8; (FLOPPY_DATA_BYTES + SECTOR) as usize],
        )
        .unwrap();

        let out_dir = tempfile::tempdir().unwrap();
        let err = build_floppy(src.path(), &out_dir.path().join("x.img"), None).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("over capacity"), "message: {msg}");
    }

    #[test]
    fn rejects_invalid_labels() {
        let src = tempfile::tempdir().unwrap();
        let out_dir = tempfile::tempdir().unwrap();
        let img = out_dir.path().join("x.img");

        let err = build_floppy(src.path(), &img, Some("TWELVECHARSX")).unwrap_err();
        assert!(err.to_string().contains("maximum is 11"));

        let err = build_floppy(src.path(), &img, Some("BAD/LABEL")).unwrap_err();
        assert!(err.to_string().contains("invalid characters"));
    }

    #[test]
    fn lowercase_label_is_uppercased_and_accepted() {
        let src = tempfile::tempdir().unwrap();
        fs::write(src.path().join("a.txt"), b"a").unwrap();
        let out_dir = tempfile::tempdir().unwrap();
        let img = out_dir.path().join("x.img");
        build_floppy(src.path(), &img, Some("bootme")).expect("lowercase label should be fine");
    }
}
