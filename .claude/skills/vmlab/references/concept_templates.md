# Templates

_Sealed qcow2 disk images in the local store, referenced by <arch>/<name>\[@<version>\]; labs boot linked clones of them._

Templates are sealed qcow2 images in the local store, referenced by
`<arch>/<name>[@<version>]` (omitting the version means latest). Labs boot
[linked clones](../references/concept_linked_clones.md) backed by them — templates are never written
to by labs. They are declared with [`template {}` blocks](../references/entity_template_block.md) and
produced by [a build](../references/concept_template_builds.md).


The store layout is `~/.local/share/vmlab/templates/<arch>/<name>/<version>/`
containing `disk.qcow2` + `template.wcl` (hardware, profile, origin, sha256
metadata). See [the template store](../references/entity_template_store.md) for the on-disk detail.


## Related

- [template {} block](../references/entity_template_block.md)

- [Template build flow](../references/concept_template_builds.md)

- [Template store](../references/entity_template_store.md)

- [Linked clones](../references/concept_linked_clones.md)

- [Scratch VMs](../references/concept_scratch_vms.md)

- [OCI distribution](../references/concept_oci.md)

- [media {} block](../references/entity_media.md)

- [The vmlab.wcl schema](../references/fact_schema_reference.md)

[← Back to SKILL.md](../SKILL.md)
