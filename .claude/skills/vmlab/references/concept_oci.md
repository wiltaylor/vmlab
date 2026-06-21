# OCI distribution

_Templates push/pull as OCI artifacts (not runnable images) through any OCI registry; chunked, multi-arch._

Templates distribute as **OCI artifacts** (not runnable container images) through
any OCI-compliant registry — GHCR, Docker Hub, Harbor, self-hosted. The local side
is always a store ref `<arch>/<name>[@<version>]`; the remote side is a normal
registry ref `host/repo:tag`.


```console
vmlab template login ghcr.io -u myuser -p <token>
vmlab template push x86_64/linux-modern@1.0 ghcr.io/owner/linux-modern:1.0
vmlab template pull ghcr.io/owner/linux-modern:1.0                # single-arch manifest
vmlab template pull ghcr.io/owner/linux-modern:1.0 --arch x86_64  # multi-arch index: --arch required
```

Login validates against the registry's `/v2/` endpoint and persists to the Docker
config (`~/.docker/config.json`); existing `docker login` credentials are reused
automatically. **Arch is never silently assumed** from the host: pulling an
ambiguous multi-arch index without `--arch` is an error. A registry ref used
directly in a lab is pulled into the store on `vmlab up` if absent, but never
re-pulled implicitly — updates are explicit via `vmlab template pull`.


```wcl
vm "box" {
  template = "ghcr.io/owner/linux-modern:1.0"
  arch     = "x86_64"     // explicit arch is required with registry refs
  memory   = "4G"
}
```

## Related

- [Templates](../references/concept_templates.md)

- [OCI artifact model](../references/fact_oci_artifact.md)

[← All concepts](../references/concepts_ref.md)
