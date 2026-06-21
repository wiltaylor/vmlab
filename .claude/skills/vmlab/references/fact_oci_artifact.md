# OCI artifact model

What a pushed template actually is in the registry:

| Aspect | Value |
| --- | --- |
| Artifact type | `application/vnd.vmlab.template.v1` (frozen; prevents `docker run` misuse) |
| Config blob | `application/vnd.vmlab.template.config.v1+json` (template metadata) |
| Layers | The qcow2 chunked into fixed-size zstd layers (`application/vnd.vmlab.template.chunk.v1+zstd`) |
| Default chunk size | **512 MiB** (clears GHCR's 10-min per-upload timeout); set via `oci_chunk_size` in host config |
| Annotations | `vnd.vmlab.template.*` record chunk count/size, total size, and whole-image digest |
| Integrity | Pull reassembles chunks in order and verifies the whole-image SHA-256 before installing |
| Multi-arch | A standard OCI image index keyed by platform arch; push the same name per arch |

## Related

- [OCI distribution](../references/concept_oci.md)

- [Templates](../references/concept_templates.md)

[← All facts](../references/facts_ref.md)
