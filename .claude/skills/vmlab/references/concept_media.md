# Media (ISO/floppy)

_media {} turns a host folder into an ISO or floppy image with a content-addressed cache; declarative, no CLI._

A `media {}` block turns a host folder into an ISO or floppy image, with a
content-addressed cache (rebuilt only when the folder changes). It is declared
inline in VM or template definitions — there is no `vmlab media` CLI.


```wcl
media { kind = "iso"    from = "./unattend/" label = "CIDATA" }
media { kind = "floppy" from = "./drivers/"  label = "DRV" }
```

## Related

- [VM block](../references/concept_vms.md)

- [Templates](../references/concept_templates.md)

[← All concepts](../references/concepts_ref.md)
