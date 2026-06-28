# Linked clones

_Labs boot copy-on-write linked clones backed by a template's sealed disk; the template itself is never written to._

A lab VM does not boot a template's disk directly — it boots a **linked clone**: a
fresh copy-on-write qcow2 whose backing file is the template's sealed `disk.qcow2`.
Writes land in the clone, so the template stays pristine and is shared read-only
across every VM and lab that references it. Clones live under `.vmlab/` beside the
lab and are deleted by `vmlab destroy` (kept by `vmlab down`).


## Related

- [Templates](../references/concept_templates.md)

- [vm {} block](../references/entity_vms.md)

- [lab {} block](../references/entity_labs.md)

[← Back to SKILL.md](../SKILL.md)
