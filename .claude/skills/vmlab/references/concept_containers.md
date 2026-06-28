# Containers

_vmlab runs unprivileged in Docker/Podman with only --device /dev/kvm; the network fabric is entirely userspace._

vmlab runs unprivileged. The container image is defined by `Containerfile`; because
vmlab builds against sibling `WCL/` and `wscript/` workspaces, the \*\*build context
is the parent directory\*\* containing all three. The image is published per release
as `ghcr.io/<owner>/vmlab:<version>`.


```console
docker build -t vmlab -f vmlab/Containerfile .   # run from the parent dir (or: just image)

docker run --rm -it --device /dev/kvm \
  -v ~/.local/share/vmlab/templates:/root/.local/share/vmlab/templates \
  -v "$PWD":/lab -w /lab vmlab vmlab up
```

`--device /dev/kvm` is the **only host grant needed** — no `--privileged`, no extra
capabilities, no host network mode (the fabric is entirely userspace). Without KVM,
vmlab falls back to slow TCG emulation with a loud warning. The entrypoint is
`vmlab` with default command `daemon start`: run long-running and drive via
`docker exec <ctr> vmlab ...`, or override the command for one-shot/CI use.


## Related

- [Daemon model](../references/concept_daemon_model.md)

- [Networking model](../references/concept_networking.md)

- [WSL2](../references/concept_wsl2.md)

[← Back to SKILL.md](../SKILL.md)
