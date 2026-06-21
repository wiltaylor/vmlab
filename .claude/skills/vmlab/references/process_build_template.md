# Build a disk template

**Purpose:** Produce a sealed, reusable qcow2 image in the local store from installer media.

_Preconditions:_ A template {} block exists in vmlab.wcl (or a file passed with -f)., Its source (ISO/qcow2/template/scratch) is reachable; URL sources have a sha256.

### Step 1: Define the template

```wcl
template "linux-modern" {
  arch = "x86_64"  version = "1.0"  profile = "linux-modern"  disk = "20G"
  source "iso" { url = "https://.../x.iso" sha256 = "abc123..." }
  media { kind = "iso" from = "./cloudinit/" label = "CIDATA" }
  nic { nat = true }
  provision "scripts/install.wscript" { }   // drives the installer; installs the guest agent
}
```

Declare `arch`, `version`, `profile` and the working `disk` size, exactly one `source` block, any build media/disks/NIC, and a provision script that drives the installer and installs the QEMU guest agent.

### Step 2: Build

```console
$ vmlab template build
$ vmlab template build -f templates.wcl linux-modern   # one named template
```

Run `vmlab template build`. vmlab resolves the source (downloads cached + content-addressed), boots a one-VM build lab, runs the provision, then shuts down, flattens and seals into the store. A failed build leaves nothing behind.

### Step 3: Confirm it sealed

```console
$ vmlab template list
```

`vmlab template list` should show `x86_64/linux-modern@1.0`. Reference it from a lab as `template = "x86_64/linux-modern"`.

> [!TIP]
> **Verification**
> `vmlab template list` shows the new `<arch>/<name>@<version>` ref, and a lab VM referencing it passes `vmlab validate`.

## Related

- [Templates](../references/concept_templates.md)

- [Build sources](../references/concept_template_sources.md)

- [Media (ISO/floppy)](../references/concept_media.md)

[← All processes](../references/processes_ref.md)
