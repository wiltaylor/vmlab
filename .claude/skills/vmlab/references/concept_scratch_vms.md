# Scratch VMs

_template = "scratch" boots a blank disk with no template; needs explicit arch, profile and disk._

A lab VM can skip templates entirely: `template = "scratch"` boots a blank disk.
It requires explicit `arch`, `profile` and `disk` size; boot media is your problem
(typically a `cdrom` plus a `media` block). Scratch disks never appear in the store
and cannot be pushed or pulled.


## Related

- [vm {} block](../references/entity_vms.md)

- [Templates](../references/concept_templates.md)

- [media {} block](../references/entity_media.md)

[← Back to SKILL.md](../SKILL.md)
