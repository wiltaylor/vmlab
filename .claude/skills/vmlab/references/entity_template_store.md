# Template store

_directory_

~/.local/share/vmlab/templates/<arch>/<name>/<version>/ — sealed disk.qcow2 + template.wcl metadata.

The local template store at
`~/.local/share/vmlab/templates/<arch>/<name>/<version>/`, each holding `disk.qcow2`
plus `template.wcl` (hardware, profile, origin, sha256 metadata). Writes are
serialised by the supervisor. Downloaded ISOs/qcow2 and built media are cached
content-addressed under `~/.cache/vmlab/artefacts/`.


## Related

- [Templates](../references/concept_templates.md)

- [OCI distribution](../references/concept_oci.md)

[← Back to SKILL.md](../SKILL.md)
