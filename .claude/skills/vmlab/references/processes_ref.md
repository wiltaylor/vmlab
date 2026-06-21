# vmlab — processes

Each process is a runbook on its own page. This is the index.

- [**Bring a lab up and tear it down**](../references/process_golden_path.md) — The everyday lifecycle: validate, boot with provisions, inspect, stop.

- [**Build a disk template**](../references/process_build_template.md) — Produce a sealed, reusable qcow2 image in the local store from installer media.

- [**Distribute a template over an OCI registry**](../references/process_distribute_oci.md) — Push a stored template to a registry and pull it on another machine.

- [**Run vmlab in a container**](../references/process_run_in_container.md) — Run a lab unprivileged inside Docker/Podman with only /dev/kvm.
