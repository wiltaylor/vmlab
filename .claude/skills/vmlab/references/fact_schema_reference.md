# The vmlab.wcl schema

A complete reference of the `vmlab.wcl` (and host `config.wcl`) schema, reflected straight from `src/config/schema.wcl` / `host_schema.wcl` with WCL's reflection builtins (`child_types` / `type_fields`) and the wdoc `type_table` component — so it can never drift from the code. Each block lists its attributes (type, whether required, description), any nested blocks, and a worked example. Descriptions are the fields' `@doc` annotations.

## `lab` block

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `name` | `utf8` | yes | Lab name (DNS label, ≤63 chars); the inline block label |
| `gui` | `bool` | no | Default for all VMs: open a VNC viewer on `up` (§11); VM `gui` overrides |

#### Child blocks

| Slot | Accepts | Multiple | Description |
| --- | --- | --- | --- |
| `segments` | `segment` | yes | Virtual L2 network segments in this lab |
| `vms` | `vm` | yes | The VMs in this lab |
| `provisions` | `provision` | yes | wscript provision scripts run on `vmlab up`, in declaration order |
| `handlers` | `on` | yes | Lifecycle event handlers (failures are logged, never fatal) |
| `records` | `record` | yes | Lab-wide static DNS entries (wildcards allowed) (§9.5) |
| `sinkholes` | `sinkhole` | yes | Lab-wide DNS sinkholes (§9.9) |

Example:

```wcl
lab "demo" {
  gui = true                       // lab-wide default: show each guest's screen
  vm "box" {
    template = "x86_64/linux-modern"
    memory   = 2GiB
    nic { nat = true }
  }
}
```

### `segment` (in `lab`)

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `name` | `utf8` | yes | Segment name (DNS label); unique per lab; the inline block label |
| `subnet` | `utf8` | no | CIDR; auto-allocated as a /24 from the host pool if omitted (§9.4) |
| `global` | `bool` | no | Owned by the supervisor and shared across labs (§9.2) |
| `dhcp` | `bool` | no | Enable DHCP (default true) (§9.4) |
| `nat` | `bool` | no | Enable NAT/internet egress for this segment (default false) (§9.7) |
| `mtu` | `i64` | no | Link MTU (576–65535); default jumbo (9000) on nat/global, else 1500 |
| `routes_to` | `list<utf8>` | no | Names of other segments to route to — daemon inter-segment routing opt-in (§9.6) |

#### Child blocks

| Slot | Accepts | Multiple | Description |
| --- | --- | --- | --- |
| `dns` | `dns` | no | DNS service override: hand out another server, or opt out (§9.5) |
| `connect` | `connect` | no | Cross-host segment peer over TCP (PSK from host config) (§9.2) |
| `routes` | `route` | yes | Guest routes pushed via DHCP option 121 (§9.6) |
| `records` | `record` | yes | Static DNS entries for this segment (wildcards allowed) (§9.5) |
| `forwards` | `forward` | yes | Host→guest port forwards (§9.8) |
| `block_rules` | `block` | yes | L3 block rules at the switch (§9.9) |
| `redirect_rules` | `redirect` | yes | L3 DNAT redirect rules (§9.9) |
| `sinkholes` | `sinkhole` | yes | DNS sinkhole rules (§9.9) |

Example:

```wcl
segment "corp" {
  subnet = "10.50.0.0/24"          // omit to auto-allocate a /24 from the host pool
  nat    = true                    // internet egress for this segment
  record { name = "dc01" ip = "10.50.0.10" }
}
```

#### `dns` (in `segment`)

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `server` | `utf8` | no | IPv4 of the DNS server to hand out via DHCP instead of the daemon |
| `enabled` | `bool` | no | Hand out a DNS server at all (default true); false suppresses the DHCP option |

Example:

```wcl
dns { server = "10.50.0.10" }      // hand out a DC as the resolver via DHCP
dns { enabled = false }            // …or suppress DNS on the segment entirely
```

#### `connect` (in `segment`)

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `host` | `utf8` | yes | Remote supervisor `host[:port]` to bridge this segment with (required) |

Example:

```wcl
connect { host = "helios:9999" }   // bridge this segment to a peer supervisor (PSK)
```

#### `route` (in `segment`)

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `dest` | `utf8` | yes | Destination CIDR, e.g. `10.60.0.0/24` (required) |
| `via` | `utf8` | yes | Gateway IPv4 the route points at (required) |

Example:

```wcl
route { dest = "10.60.0.0/24" via = "10.50.0.254" }   // pushed via DHCP option 121
```

#### `record` (in `segment`)

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `name` | `utf8` | yes | DNS name to resolve; wildcards allowed, e.g. `*.internal` (required) |
| `ip` | `utf8` | yes | IPv4 address the name resolves to (required) |

Example:

```wcl
record { name = "srv" ip = "10.50.0.5" }     // wildcards OK: name = "*.internal"
```

#### `forward` (in `segment`)

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `host_port` | `i64` | yes | Host port to listen on (1–65535); unique across the lab (required) |
| `to` | `utf8` | yes | Target as `vm:port`; the VM must be declared (required) |
| `proto` | `utf8` | no | Protocol: `tcp` (default) \| `udp` \| `both` |

Example:

```wcl
forward { host_port = 13389 to = "dc01:3389" proto = "tcp" }
```

#### `block` (in `segment`)

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `cidr` | `utf8` | yes | IPv4 CIDR to drop traffic to/from (required) |
| `proto` | `utf8` | no | Protocol to scope the rule: `tcp` \| `udp` \| `icmp` |
| `port` | `i64` | no | Port to scope the rule (1–65535); requires `proto` |

Example:

```wcl
block { cidr = "192.0.2.0/24" proto = "tcp" port = 443 }
```

#### `redirect` (in `segment`)

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `from` | `utf8` | yes | Match destination as `ip[:port]` (required) |
| `to` | `utf8` | yes | Rewrite destination to `ip[:port]` (required) |
| `proto` | `utf8` | no | Protocol to scope the rule: `tcp` \| `udp` |

Example:

```wcl
redirect { from = "10.50.0.254:53" to = "10.50.0.10:53" proto = "udp" }
```

#### `sinkhole` (in `segment`)

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `pattern` | `utf8` | yes | DNS name pattern to sink; wildcards allowed (required) |
| `mode` | `utf8` | no | Response: `nxdomain` (default) \| `zero` (resolve to 0.0.0.0) |

Example:

```wcl
sinkhole { pattern = "*.telemetry.com" mode = "nxdomain" }   // or mode = "zero"
```

### `vm` (in `lab`)

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `name` | `utf8` | yes | VM name (DNS label); unique per lab; the inline block label |
| `template` | `utf8` | yes | `<arch>/<name>[@<version>]`, `scratch`, or an OCI registry ref (required) |
| `arch` | `utf8` | no | Architecture; required for `scratch` and registry references |
| `profile` | `utf8` | no | Guest OS profile (hardware defaults); required for `scratch` |
| `cpus` | `i64` | no | vCPU count (> 0); inherited from template→profile if omitted |
| `memory` | `std.ByteSize` | no | RAM as a byte size, e.g. `8GiB`/`512MiB`; inherited if omitted |
| `disk` | `std.ByteSize` | no | Primary disk size, e.g. `64GiB` — scratch VMs only (rejected on cloned VMs) |
| `cdrom` | `utf8` | no | Path to an ISO to attach as a CD-ROM (relative to lab root) |
| `floppy` | `utf8` | no | Path to a floppy image to attach (relative to lab root) |
| `depends_on` | `list<utf8>` | no | VM names to wait for before this one (no cycles) |
| `nested` | `bool` | no | Enable nested virtualisation (host CPU passthrough) |
| `gui` | `bool` | no | Open a VNC viewer on `up` (§11); the VM always runs headless |
| `display` | `utf8` | no | QEMU display string; inherited from template→profile if omitted |
| `firmware` | `utf8` | no | Firmware: `ovmf` \| `seabios`; inherited from template→profile |
| `tpm` | `bool` | no | Enable a TPM 2.0 device; inherited from template→profile |
| `secure_boot` | `bool` | no | Enable secure boot (OVMF only); inherited from template→profile |
| `qemu_args` | `list<utf8>` | no | Raw QEMU flags appended last — escape hatch (§5.2) |

#### Child blocks

| Slot | Accepts | Multiple | Description |
| --- | --- | --- | --- |
| `gpu` | `gpu` | no | GPU acceleration (passthrough / virgl / vulkan) |
| `nics` | `nic` | yes | Network interfaces; no NICs = air-gapped (shares need ≥1 NIC) |
| `extra_disks` | `disk` | yes | Additional disks beyond the primary disk |
| `shares` | `share` | yes | SMB shared folders (require ≥1 NIC) (§7.5) |
| `media` | `media` | yes | ISO/floppy images built from a folder (§6.3) |

Example:

```wcl
vm "dc01" {
  template = "x86_64/windows-2025"
  cpus     = 4
  memory   = 8GiB
  nic   { segment = "corp" ip = "10.50.0.10" }
  share { host = "./src" guest = "D:\\src" }
}
```

#### `gpu` (in `vm`)

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `mode` | `utf8` | yes | Mode: `passthrough` \| `virgl` \| `vulkan` (required) |
| `address` | `utf8` | no | Host PCI address, e.g. `0000:01:00.0` — required for `passthrough` |

Example:

```wcl
gpu { mode = "passthrough" address = "0000:01:00.0" }   // or mode = "virgl" | "vulkan"
```

#### `nic` (in `vm`)

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `segment` | `utf8` | no | Segment name to attach to; required unless `nat = true` |
| `nat` | `bool` | no | Shorthand: attach to the per-lab built-in NAT segment (§9.7) |
| `ip` | `utf8` | no | Static IPv4 (becomes a DHCP reservation); must be in the subnet, unique |
| `mac` | `utf8` | no | Fixed MAC, e.g. `52:54:00:ab:cd:ef`; generated and persisted otherwise |
| `isolated` | `bool` | no | Port isolation: reach gateway/forwards but not segment neighbours (§9.1) |

Example:

```wcl
nic { segment = "corp" ip = "10.50.0.10" mac = "52:54:00:aa:bb:cc" }
nic { nat = true }                       // per-lab built-in NAT segment shorthand
nic { segment = "dmz" isolated = true }  // port isolation
```

#### `disk` (in `vm`)

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `name` | `utf8` | yes | Disk identifier; the inline block label |
| `size` | `std.ByteSize` | no | Blank disk size, e.g. `10GiB`; one of `size`/`from` is required |
| `from` | `utf8` | no | Folder copied onto a fresh FAT filesystem; one of `size`/`from` is required |

Example:

```wcl
disk "data"      { size = 10GiB }         // extra blank disk
disk "formatted" { from = "./payload/" }  // folder copied onto a fresh FAT filesystem
```

#### `share` (in `vm`)

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `host` | `utf8` | yes | Host directory to share; must exist (required) |
| `guest` | `utf8` | yes | Guest mount path, e.g. `/mnt/src` or `D:\data` (required) |
| `readonly` | `bool` | no | Mount read-only (default false) |
| `smb1` | `bool` | no | Enable the SMB1 dialect + auth relaxation for XP/2003-era guests |
| `name` | `utf8` | no | SMB share name; derived from the guest path if omitted |

Example:

```wcl
share { host = "./src"  guest = "/mnt/src" }
share { host = "~/data" guest = "D:\\data" readonly = true }
share { host = "./old"  guest = "X:" smb1 = true }   // legacy dialect for XP/2003
```

#### `media` (in `vm`)

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `kind` | `utf8` | yes | Image kind: `iso` \| `floppy` (required) |
| `from` | `utf8` | yes | Source folder built into the image; must exist (required) |
| `label` | `utf8` | no | Volume label for the image |

Example:

```wcl
media { kind = "iso"    from = "./unattend/" label = "CIDATA" }
media { kind = "floppy" from = "./drivers/"  label = "DRV" }
```

### `provision` (in `lab`)

Provision script run during `vmlab up` (§10.4). Optional vms list scopes
the script for depends_on satisfaction (§7.2).

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `script` | `utf8` | yes | Path to the `.ws` file; must exist and compile; the inline label |
| `vms` | `list<utf8>` | no | VM names this script is scoped to (gates their `depends_on`) (§7.2) |

Example:

```wcl
provision "scripts/setup.ws" { }                     // runs on `vmlab up`, in order
provision "scripts/join.ws"  { vms = ["client01"] }  // scoped: gates depends_on
```

### `on` (in `lab`)

Event handler binding (§8.2).

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `event` | `utf8` | yes | Event name to handle, e.g. `vm.crashed`; the inline block label |
| `run` | `utf8` | yes | Path to the handler `.ws` file; must exist and compile (required) |

Example:

```wcl
on "vm.crashed"    { run = "scripts/collect-dumps.ws" }
on "host.disk_low" { run = "scripts/alert.ws" }
```

### `record` (in `lab`)

Static DNS entry (wildcards allowed in name) (§9.5).

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `name` | `utf8` | yes | DNS name to resolve; wildcards allowed, e.g. `*.internal` (required) |
| `ip` | `utf8` | yes | IPv4 address the name resolves to (required) |

Example:

```wcl
record { name = "srv" ip = "10.50.0.5" }     // wildcards OK: name = "*.internal"
```

### `sinkhole` (in `lab`)

DNS sinkhole (§9.9): NXDOMAIN by default, or 0.0.0.0 with mode = "zero".

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `pattern` | `utf8` | yes | DNS name pattern to sink; wildcards allowed (required) |
| `mode` | `utf8` | no | Response: `nxdomain` (default) \| `zero` (resolve to 0.0.0.0) |

Example:

```wcl
sinkhole { pattern = "*.telemetry.com" mode = "nxdomain" }   // or mode = "zero"
```

## `template` block

Template definition (§6.1), buildable with `vmlab template build`.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `name` | `utf8` | yes | Template name, e.g. `linux-modern`; the inline block label |
| `arch` | `utf8` | yes | Architecture — selects the QEMU system emulator (required) |
| `version` | `utf8` | yes | Version string, non-empty; name+arch+version is unique (required) |
| `registry` | `utf8` | no | Full OCI repo to publish to / version-bump against (§6.4) |
| `profile` | `utf8` | no | Guest OS profile (hardware defaults) for the build VM |
| `cpus` | `i64` | no | vCPU count for the build VM; inherited by clones |
| `memory` | `std.ByteSize` | no | RAM for the build VM, e.g. `8GiB`; inherited by clones |
| `disk` | `std.ByteSize` | no | Working disk size for the build, e.g. `64GiB`; required for `scratch` source |
| `display` | `utf8` | no | QEMU display string for the build VM |
| `firmware` | `utf8` | no | Firmware: `ovmf` \| `seabios` |
| `tpm` | `bool` | no | Enable a TPM 2.0 device |
| `secure_boot` | `bool` | no | Enable secure boot (OVMF only) |
| `nested` | `bool` | no | Enable nested virtualisation for the build VM |
| `gui` | `bool` | no | Watch the build VM via a VNC viewer (§11) |
| `qemu_args` | `list<utf8>` | no | Raw QEMU flags for the build VM — escape hatch (§5.2) |
| `first_boot` | `utf8` | no | wscript run on first instantiation of a clone, before ready |

#### Child blocks

| Slot | Accepts | Multiple | Description |
| --- | --- | --- | --- |
| `source` | `source` | no | What the build starts from — exactly one of four forms (required) |
| `media` | `media` | yes | ISO/floppy images attached to the build (§6.3) |
| `provisions` | `provision` | yes | Provision scripts that drive the build |
| `nics` | `nic` | yes | NICs for the build VM (optional; the build VM may be air-gapped) |
| `extra_disks` | `disk` | yes | Additional disks attached during the build |

Example:

```wcl
template "linux-modern" {
  arch    = "x86_64"
  version = "1.0"
  profile = "linux-modern"
  disk    = 20GiB                  // working disk size for the build
  source "iso" { url = "https://releases.ubuntu.com/.../x.iso" sha256 = "abc123…" }
  provision "scripts/install.ws" { }
}
```

### `source` (in `template`)

Template build source (§6.1): exactly one of the four forms.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `kind` | `utf8` | yes | Source kind: `iso` \| `qcow2` \| `template` \| `scratch`; the inline label |
| `path` | `utf8` | no | Local file path — `iso`/`qcow2`; mutually exclusive with `url` |
| `url` | `utf8` | no | Remote artefact URL — `iso`/`qcow2`; requires `sha256` |
| `sha256` | `utf8` | no | SHA-256 of the remote artefact; required with `url` |
| `from` | `utf8` | no | Source template `<arch>/<name>[@<version>]` — kind `template` (layered build) |

Example:

```wcl
source "iso"      { path = "./isos/win11.iso" }           // local installer ISO
source "iso"      { url = "https://…" sha256 = "…" }      // downloaded + verified
source "qcow2"    { path = "./base.qcow2" }               // existing disk as base
source "template" { from = "x86_64/linux-modern@1.0" }    // layered build
source "scratch"  { }                                     // blank disk
```

### `media` (in `template`)

ISO/floppy image built from a folder (§6.3).

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `kind` | `utf8` | yes | Image kind: `iso` \| `floppy` (required) |
| `from` | `utf8` | yes | Source folder built into the image; must exist (required) |
| `label` | `utf8` | no | Volume label for the image |

Example:

```wcl
media { kind = "iso"    from = "./unattend/" label = "CIDATA" }
media { kind = "floppy" from = "./drivers/"  label = "DRV" }
```

### `provision` (in `template`)

Provision script run during `vmlab up` (§10.4). Optional vms list scopes
the script for depends_on satisfaction (§7.2).

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `script` | `utf8` | yes | Path to the `.ws` file; must exist and compile; the inline label |
| `vms` | `list<utf8>` | no | VM names this script is scoped to (gates their `depends_on`) (§7.2) |

Example:

```wcl
provision "scripts/setup.ws" { }                     // runs on `vmlab up`, in order
provision "scripts/join.ws"  { vms = ["client01"] }  // scoped: gates depends_on
```

### `nic` (in `template`)

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `segment` | `utf8` | no | Segment name to attach to; required unless `nat = true` |
| `nat` | `bool` | no | Shorthand: attach to the per-lab built-in NAT segment (§9.7) |
| `ip` | `utf8` | no | Static IPv4 (becomes a DHCP reservation); must be in the subnet, unique |
| `mac` | `utf8` | no | Fixed MAC, e.g. `52:54:00:ab:cd:ef`; generated and persisted otherwise |
| `isolated` | `bool` | no | Port isolation: reach gateway/forwards but not segment neighbours (§9.1) |

Example:

```wcl
nic { segment = "corp" ip = "10.50.0.10" mac = "52:54:00:aa:bb:cc" }
nic { nat = true }                       // per-lab built-in NAT segment shorthand
nic { segment = "dmz" isolated = true }  // port isolation
```

### `disk` (in `template`)

Additional disk (§5.2): blank by size, or pre-formatted from a folder.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `name` | `utf8` | yes | Disk identifier; the inline block label |
| `size` | `std.ByteSize` | no | Blank disk size, e.g. `10GiB`; one of `size`/`from` is required |
| `from` | `utf8` | no | Folder copied onto a fresh FAT filesystem; one of `size`/`from` is required |

Example:

```wcl
disk "data"      { size = 10GiB }         // extra blank disk
disk "formatted" { from = "./payload/" }  // folder copied onto a fresh FAT filesystem
```

## `host` block

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `subnet_pool` | `utf8` | no | Segment auto-allocation pool (CIDR); default `10.213.0.0/16` (§9.4) |
| `dns_suffix` | `utf8` | no | Suffix for auto-registered VM names; default `vmlab.internal` (§9.5) |
| `dns_upstream` | `utf8` | no | Upstream resolver `ip[:port]`; default: the host resolver |
| `disk_low_percent` | `i64` | no | `host.disk_low` watchdog threshold percent (0–100); default 10 (§8.1) |
| `psk` | `utf8` | no | Pre-shared key for cross-host segment links (§9.2) |
| `viewer` | `utf8` | no | VNC viewer command; `{}` is replaced by the target (§11) |
| `oci_chunk_size` | `std.ByteSize` | no | OCI layer chunk size for template push; default `512MiB` (§6.4) |

Example:

```wcl
host {
  subnet_pool      = "10.213.0.0/16"   // segment auto-allocation pool (default shown)
  dns_suffix       = "vmlab.internal"
  dns_upstream     = "1.1.1.1"
  disk_low_percent = 10
  viewer           = "vncviewer {}"    // {} = target
  oci_chunk_size   = 512MiB
}
```

## Related

- [lab {} block](../references/entity_labs.md)

- [vm {} block](../references/entity_vms.md)

- [Networking model](../references/concept_networking.md)

- [Templates](../references/concept_templates.md)

- [Host config](../references/concept_host_config.md)

- [What `vmlab validate` checks](../references/fact_validate_checks.md)

[← Back to SKILL.md](../SKILL.md)
