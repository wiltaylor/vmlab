# Templates

_Sealed qcow2 disk images in the local store, referenced by <arch>/<name>\[@<version>\]; labs boot linked clones of them._

Templates are sealed qcow2 images in the local store, referenced by
`<arch>/<name>[@<version>]` (omitting the version means latest). Labs boot \*\*linked
clones\*\* backed by them — templates are never written to by labs. `template {}`
blocks live in `vmlab.wcl` (or any file passed with `-f`).


```wcl
import <vmlab.wcl>

template "linux-modern" {
  arch    = "x86_64"           // required — selects the QEMU system emulator
  version = "1.0"              // required
  profile = "linux-modern"     // hardware defaults
  cpus    = 4                  // optional hardware overrides (also memory, display,
  memory  = "8G"               // firmware, tpm, secure_boot, nested, qemu_args)
  disk    = "20G"              // working disk size for the build
  gui     = true               // watch the build VM's screen in a QEMU window

  source "iso" { url = "https://releases.ubuntu.com/.../x.iso" sha256 = "abc123..." }

  media { kind = "iso" from = "./cloudinit/" label = "CIDATA" }   // built from folder, cached
  nic { nat = true }                                              // build VM network access
  disk "extra" { size = "10G" }                                   // extra disks during build
  provision "scripts/install.wscript" { }                         // drives the installer
}
```

## Build flow

`vmlab template build` resolves the source (URL downloads are cached and
content-addressed under `~/.cache/vmlab/artefacts/`), creates a working qcow2,
synthesises a one-VM lab from the template definition, boots it per the hardware
profile, runs the provision scripts (full wscript API — keystrokes, screen matching,
exec; the script should install the QEMU guest agent), shuts down gracefully,
flattens and seals into the store. **A failed build leaves nothing behind.** The
store layout is `~/.local/share/vmlab/templates/<arch>/<name>/<version>/` containing
`disk.qcow2` + `template.wcl` (hardware, profile, origin, sha256 metadata).


## Related

- [Build sources](../references/concept_template_sources.md)

- [Scratch VMs](../references/concept_scratch_vms.md)

- [OCI distribution](../references/concept_oci.md)

- [Media (ISO/floppy)](../references/concept_media.md)

[← All concepts](../references/concepts_ref.md)
