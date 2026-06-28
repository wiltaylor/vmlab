# Running vmlab in Docker

The image (`Containerfile` at the repo root) ships both the `vmlab` CLI and the
`vmlab-web` UI server. By default a container runs the web UI on port `7878`.

## Quick start (compose)

```sh
docker compose up --build
```

Then open <http://localhost:7878> and sign in. Credentials come from the
environment (defaults `vmlab` / `vmlab`); override them:

```sh
VMLAB_WEB_USER=me VMLAB_WEB_PASSWORD=s3cret docker compose up --build
```

> The web UI binds `0.0.0.0` inside the container so the published port works,
> and vmlab-web requires a login on any non-loopback bind. **Change the default
> password before exposing this beyond your machine.**

## Your lab

Put your `vmlab.wcl` in `docker/lab/` — it's bind-mounted to `/lab` in the
container, so edit it on the host with your normal editor. A sample lab is
provided (`docker/lab/vmlab.wcl`, an Alpine VM from a public OCI template).
Templates are pulled on the first `up` into the persistent `vmlab-templates`
volume.

KVM acceleration uses the host's `/dev/kvm` (mapped in `compose.yaml`); remove
that device mapping to fall back to slower TCG emulation.

## Plain `docker run`

```sh
# Web UI
docker run --rm -p 7878:7878 --device /dev/kvm \
  -e VMLAB_WEB_USER=admin -e VMLAB_WEB_PASSWORD=secret \
  -v "$PWD":/lab vmlab:latest

# CLI (override the default command)
docker run --rm --device /dev/kvm -v "$PWD":/lab vmlab:latest vmlab up
```
