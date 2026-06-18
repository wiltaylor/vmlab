# alpine-registry — pull a template from an OCI registry on `up`

A one-VM lab whose template is referenced by **OCI registry ref** instead of a
local store ref. It demonstrates vmlab's on-demand template distribution
(PRD §6.4): there is no `vmlab template build` or `vmlab template pull` step —
the template is fetched from the registry the first time you bring the lab up.

The VM block points at a published template:

```wcl
vm "alp" {
  template = "ghcr.io/vmlabdev/vmlab-templates/alpine-3.23"   // → :latest
  arch     = "x86_64"
  ...
}
```

The ref is `host/owner/[group/]name[:tag]`; every version of a template lives
under one package as a tag. With no tag it tracks the moving **`latest`**
(newest stable); `:latest-prerelease` tracks pre-releases and `:<version>`
pins one. On `up`, the per-lab daemon resolves the tag and, if that version is
absent locally, pulls and installs it, then boots; a cached version is reused
without re-downloading.

## Run

```sh
vmlab up                        # first run pulls the template, then boots
ssh vmlab@localhost -p 12222    # password: vmlab
vmlab down
```

The first `up` takes longer than usual because of the pull (a few tens of
seconds for this ~small Alpine template, depending on your connection).
`vmlab status` shows the VM with its registry ref under `TEMPLATE`.

## Cleaning up the cached template

The pull leaves the template in your local store. To force the next `up` to
pull again, remove the cached version (find it with `vmlab template list`):

```sh
vmlab template rm x86_64/alpine-3.23@<version> --force
```
