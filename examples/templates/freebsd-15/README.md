# FreeBSD 15.0-RELEASE template

Builds `x86_64/freebsd-15` into the local template store from the official
FreeBSD **BASIC-CLOUDINIT** cloud image (UFS). FreeBSD is the one BSD that
fits vmlab's guest-agent model cleanly: `qemu-guest-agent` is packaged and its
core commands (`guest-ping`, `guest-exec`) work, and FreeBSD's native
cloud-init (`nuageinit`) reads the same CIDATA NoCloud seed as the Linux
templates.

```sh
vmlab validate
vmlab template build
vmlab template list      # → x86_64/freebsd-15@15.0
```

What happens (`scripts/install.wscript` narrates it in the build log):

1. The build VM boots the cloud image with `cloudinit/` attached as a
   `CIDATA` volume (NoCloud datasource, read by nuageinit).
2. nuageinit creates user `vmlab` (password `vmlab`, `wheel` group), then
   `pkg`-installs `qemu-guest-agent` and enables it
   (`sysrc qemu_guest_agent_enable=YES` + `service qemu-guest-agent start`).
3. The script waits for the guest agent to respond.
4. vmlab powers the VM off and seals the disk into the store.

Notes:
- The image ships **xz-compressed** (`.qcow2.xz`); vmlab verifies the
  download's sha256 then decompresses it.
- FreeBSD cloud images are **UEFI-only**, so the template uses
  `firmware = "ovmf"`.
- Some guest-agent commands (suspend, fsfreeze, vcpu/memory hotplug) are
  disabled in the FreeBSD port; vmlab doesn't use them.

Bumps: update `version`, the image filename in `url`, and `sha256` (from
`CHECKSUM.SHA256` beside the image under
<https://download.freebsd.org/releases/VM-IMAGES/15.0-RELEASE/amd64/Latest/>).
