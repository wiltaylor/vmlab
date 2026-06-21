# Labs

_A lab is a set of VMs plus the virtual networks connecting them, declared in vmlab.wcl._

A **lab** is declared in `vmlab.wcl`, located by walking up from the current
directory (like git). Lab-local working data — linked clones, snapshots, built
media, screenshots — lives in `.vmlab/` beside it; gitignore that directory.
Every WCL file in a lab starts with `import <vmlab.wcl>`.


## Minimal lab

```wcl
import <vmlab.wcl>

lab "demo" {
  gui = true                  // lab-wide default: show each guest's screen in a
                              // QEMU window (VM `gui` overrides; silently headless
                              // when no display server is available)
  vm "box" {
    template = "x86_64/linux-modern"
    memory   = "2G"
    nic { nat = true }
  }
}
```

Closing a `gui` window kills that VM (QEMU semantics) — it surfaces as
`vm.crashed`. The VNC socket (`vmlab console`) is available either way.


## Examples

### A small multi-segment lab

Two segments (one with the DC as DNS, one zero-config), three VMs with static and dynamic leases, a scoped provision and crash handlers.

```wcl
import <vmlab.wcl>

lab "ad-lab" {
  segment "corp" {
    subnet = "10.50.0.0/24"
    dns { server = "10.50.0.10" }                 // AD owns DNS
    route { dest = "10.60.0.0/24" via = "10.50.0.254" }
  }
  segment "dmz" { }                               // zero-config: auto subnet, daemon DHCP/DNS

  vm "dc01" {
    template = "x86_64/windows-server-2025"
    cpus = 4  memory = "8G"
    nic { segment = "corp" ip = "10.50.0.10" }    // static → DHCP reservation
  }
  vm "client01" {
    template   = "x86_64/windows-11@26100.1"      // version pin
    depends_on = ["dc01"]                         // boot ordering
    nic { segment = "corp" }                      // dynamic lease
  }
  vm "buildbox" {
    template = "x86_64/linux-modern"
    nic { nat = true }
  }

  provision "scripts/setup.wscript" { vms = ["dc01"] }
  on "vm.crashed"    { run = "scripts/collect-dumps.wscript" }
  on "host.disk_low" { run = "scripts/alert.wscript" }
}
```

## Related

- [VM block](../references/concept_vms.md)

- [Networking & segments](../references/concept_networking.md)

- [Provisions & event handlers](../references/concept_provisions.md)

- [Daemon model](../references/concept_daemon_model.md)

[← All concepts](../references/concepts_ref.md)
