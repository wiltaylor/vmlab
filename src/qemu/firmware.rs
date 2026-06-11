//! Locate firmware blobs (OVMF/SeaBIOS/AAVMF) across distro layouts.
//! Exact paths vary per distribution; we search the well-known spots and
//! fail with an actionable error listing what was tried.

use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};

/// A resolved UEFI firmware pair: read-only CODE image plus a pristine VARS
/// template to copy per VM.
#[derive(Debug, Clone)]
pub struct UefiFirmware {
    pub code: PathBuf,
    pub vars_template: PathBuf,
}

fn first_existing(candidates: &[&str]) -> Option<PathBuf> {
    candidates
        .iter()
        .map(Path::new)
        .find(|p| p.is_file())
        .map(Path::to_path_buf)
}

/// OVMF for x86_64. `secure_boot` selects the secboot build (which requires
/// the matching 4m VARS).
pub fn ovmf_x86_64(secure_boot: bool) -> Result<UefiFirmware> {
    let code_candidates: &[&str] = if secure_boot {
        &[
            "/usr/share/edk2/x64/OVMF_CODE.secboot.4m.fd",
            "/usr/share/edk2/ovmf/OVMF_CODE.secboot.fd",
            "/usr/share/OVMF/OVMF_CODE_4M.secboot.fd",
            "/usr/share/OVMF/OVMF_CODE.secboot.fd",
            "/usr/share/edk2-ovmf/OVMF_CODE.secboot.fd",
        ]
    } else {
        &[
            "/usr/share/edk2/x64/OVMF_CODE.4m.fd",
            "/usr/share/edk2/ovmf/OVMF_CODE.fd",
            "/usr/share/OVMF/OVMF_CODE_4M.fd",
            "/usr/share/OVMF/OVMF_CODE.fd",
            "/usr/share/edk2-ovmf/OVMF_CODE.fd",
            "/usr/share/qemu/ovmf-x86_64-code.bin",
        ]
    };
    let vars_candidates: &[&str] = &[
        "/usr/share/edk2/x64/OVMF_VARS.4m.fd",
        "/usr/share/edk2/ovmf/OVMF_VARS.fd",
        "/usr/share/OVMF/OVMF_VARS_4M.fd",
        "/usr/share/OVMF/OVMF_VARS.fd",
        "/usr/share/edk2-ovmf/OVMF_VARS.fd",
        "/usr/share/qemu/ovmf-x86_64-vars.bin",
    ];
    let code = first_existing(code_candidates).ok_or_else(|| {
        anyhow!(
            "OVMF firmware not found; tried: {}",
            code_candidates.join(", ")
        )
    })?;
    let vars_template = first_existing(vars_candidates).ok_or_else(|| {
        anyhow!(
            "OVMF VARS template not found; tried: {}",
            vars_candidates.join(", ")
        )
    })?;
    Ok(UefiFirmware {
        code,
        vars_template,
    })
}

/// UEFI for aarch64 (QEMU_EFI / AAVMF).
pub fn uefi_aarch64() -> Result<UefiFirmware> {
    let code_candidates: &[&str] = &[
        "/usr/share/edk2/aarch64/QEMU_CODE.fd",
        "/usr/share/edk2/aarch64/QEMU_EFI.fd",
        "/usr/share/AAVMF/AAVMF_CODE.fd",
        "/usr/share/qemu-efi-aarch64/QEMU_EFI.fd",
    ];
    let vars_candidates: &[&str] = &[
        "/usr/share/edk2/aarch64/QEMU_VARS.fd",
        "/usr/share/AAVMF/AAVMF_VARS.fd",
    ];
    let code = first_existing(code_candidates).ok_or_else(|| {
        anyhow!(
            "aarch64 UEFI firmware not found; tried: {}",
            code_candidates.join(", ")
        )
    })?;
    let vars_template = first_existing(vars_candidates).ok_or_else(|| {
        anyhow!(
            "aarch64 UEFI VARS template not found; tried: {}",
            vars_candidates.join(", ")
        )
    })?;
    Ok(UefiFirmware {
        code,
        vars_template,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_has_ovmf() {
        // This host (and the official container image) ships edk2.
        let fw = ovmf_x86_64(false).unwrap();
        assert!(fw.code.is_file());
        assert!(fw.vars_template.is_file());
        let sb = ovmf_x86_64(true).unwrap();
        assert!(sb.code.to_string_lossy().contains("secboot"));
    }
}
