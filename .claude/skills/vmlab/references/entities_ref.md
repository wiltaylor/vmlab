# vmlab — entities

Each entity has its own page. This is the index.

- [**Lab**](../references/entity_lab_api.md) — _wscript API object_ — The lab handle passed to fn main(lab: Lab) / fn handle(event, lab) — find VMs and segments, log.

- [**Vm**](../references/entity_vm_api.md) — _wscript API object_ — A VM handle: lifecycle, state, snapshots, keyboard/mouse, screen matching/OCR, and the guest agent.

- [**Segment**](../references/entity_seg_api.md) — _wscript API object_ — Runtime network-rule mutation: DNS, blocking, redirects and forwards on a live segment.

- [**Match / ExecResult / Event**](../references/entity_match_type.md) — _wscript data type_ — The struct return types: Match (image/OCR hit), ExecResult (guest exec), Event (handler payload).

- [**vmlab.wcl**](../references/entity_vmlab_wcl.md) — _file_ — The lab definition, found by walking up from the current directory (like git).

- [**.vmlab/**](../references/entity_dot_vmlab.md) — _directory_ — Lab-local working data beside vmlab.wcl: linked clones, snapshots, built media, screenshots, SMB creds. Gitignore it.

- [**Template store**](../references/entity_template_store.md) — _directory_ — ~/.local/share/vmlab/templates/<arch>/<name>/<version>/ — sealed disk.qcow2 + template.wcl metadata.
