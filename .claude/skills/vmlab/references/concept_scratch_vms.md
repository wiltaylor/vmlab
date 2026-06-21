# Scratch VMs

_template = "scratch" boots a blank disk with no template; needs explicit arch, profile and disk._

A lab VM can skip templates entirely: `template = "scratch"` boots a blank disk.
It requires explicit `arch`, `profile` and `disk` size; boot media is your problem
(typically a `cdrom` plus a `media` block). Scratch disks never appear in the store
and cannot be pushed or pulled.


## Related

- [VM block](../references/concept_vms.md)

- [Templates](../references/concept_templates.md)

- [Media (ISO/floppy)](../references/concept_media.md)

[← All concepts](../references/concepts_ref.md)
