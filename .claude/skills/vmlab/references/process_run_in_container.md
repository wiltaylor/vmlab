# Run vmlab in a container

## Purpose

Run a lab unprivileged inside Docker/Podman with only /dev/kvm.

## Prerequisites

- The host exposes /dev/kvm (else vmlab falls back to slow TCG).
- The vmlab image is built or pulled.

## Flowchart

![diagram](../_wdoc/process_run_in_container-diagram-1.svg)

## Steps

### Step 1: Build the image

```console
$ docker build -t vmlab -f vmlab/Containerfile .   # from the PARENT dir (or: just image)
```

> [!NOTE]
> **Build context**
> vmlab builds against sibling WCL/ and wscript/ workspaces, so the build context is the parent directory containing all three.

Build from the parent directory (or run `just image` from inside vmlab/). The image is also published per release as `ghcr.io/<owner>/vmlab:<version>`.

### Step 2: Run a lab

```console
$ docker run --rm -it --device /dev/kvm \
    -v ~/.local/share/vmlab/templates:/root/.local/share/vmlab/templates \
    -v "$PWD":/lab -w /lab vmlab vmlab up
```

> [!TIP]
> **Only /dev/kvm**
> No --privileged, no extra capabilities, no host network mode — the fabric is entirely userspace.

Mount the template store (persistent) and the lab directory, grant `--device /dev/kvm`, and run a vmlab verb. For long-running use, start with the default `daemon start` CMD and drive via `docker exec <ctr> vmlab ...`.

> [!TIP]
> **Verification**
> `vmlab status` (via `docker exec` or in the one-shot command) reports the lab running; no KVM-fallback warning appears in the logs.

## Related

- [Containers](../references/concept_containers.md)

- [WSL2](../references/concept_wsl2.md)

- [Daemon model](../references/concept_daemon_model.md)

[← Back to SKILL.md](../SKILL.md)
