# vmlab — concepts

Each concept has its own page. This is the index.

- [**Labs**](../references/concept_labs.md) — A lab is a set of VMs plus the virtual networks connecting them, declared in vmlab.wcl.

- [**VM block**](../references/concept_vms.md) — Each vm {} declares a guest: its template, hardware, NICs, disks, shares and media.

- [**Networking & segments**](../references/concept_networking.md) — Virtual L2 segments with daemon DHCP/DNS, NAT, routing, port forwards and L3 filtering — all declarative.

- [**SMB shares**](../references/concept_shares.md) — share {} mounts a host folder into a guest over SMB, served by the lab daemon at the segment gateway.

- [**Provisions & event handlers**](../references/concept_provisions.md) — provision {} scripts run on `vmlab up`; on "event" {} handlers react to lifecycle events.

- [**Templates**](../references/concept_templates.md) — Sealed qcow2 disk images in the local store, referenced by <arch>/<name>\[@<version>\]; labs boot linked clones of them.

- [**Build sources**](../references/concept_template_sources.md) — Exactly one source {} block per template selects what the build starts from: iso, qcow2, template, or scratch.

- [**Scratch VMs**](../references/concept_scratch_vms.md) — template = "scratch" boots a blank disk with no template; needs explicit arch, profile and disk.

- [**Media (ISO/floppy)**](../references/concept_media.md) — media {} turns a host folder into an ISO or floppy image with a content-addressed cache; declarative, no CLI.

- [**OCI distribution**](../references/concept_oci.md) — Templates push/pull as OCI artifacts (not runnable images) through any OCI registry; chunked, multi-arch.

- [**Daemon model**](../references/concept_daemon_model.md) — A two-tier daemon: the supervisor vmlabd (one per user) plus one lab daemon per running lab, auto-started by the CLI.

- [**Host config**](../references/concept_host_config.md) — Optional ~/.config/vmlab/config.wcl tunes the subnet pool, DNS, disk watchdog, viewer command and OCI chunk size.

- [**Guest OS profiles**](../references/concept_profiles.md) — Shipped hardware-default sets (windows-11, linux-modern, …); override or extend by dropping \*.wcl into ~/.config/vmlab/profiles/.

- [**Containers & WSL2**](../references/concept_containers.md) — vmlab runs unprivileged in Docker/Podman and on WSL2 with only --device /dev/kvm; the network fabric is entirely userspace.

- [**wscript: overview**](../references/concept_wscript_overview.md) — A statically typed, Rust-flavoured scripting language; vmlab type-checks scripts at `vmlab validate` time.

- [**wscript: types & values**](../references/concept_wscript_types.md) — 64-bit ints, floats, strings, bool; no implicit numeric conversion, no truthiness, reference semantics for compound types.

- [**wscript: functions & control flow**](../references/concept_wscript_functions.md) — Block-valued fn bodies, closures, if/for/while/loop; ranges are exclusive (0..10) or inclusive (0..=10).

- [**wscript: pattern matching & errors**](../references/concept_wscript_matching.md) — Option\[T\] and Result\[T,E\] are built in; let-else, match (exhaustive), and ? are the idioms vmlab scripts live on.

- [**wscript: containers, strings & modules**](../references/concept_wscript_containers.md) — List and Map builtins, immutable strings, the always-on prelude, and `use vmlab`; scripts are single files in v1.

- [**wscript: not in v1**](../references/concept_wscript_limits.md) — No references, lifetimes, user generics, exceptions, async, threads, truthiness, implicit conversions, interpolation, += or bitwise ops.
