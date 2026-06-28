# Vm: snapshot methods

| Method | Returns |
| --- | --- |
| `vm.snapshot(name: string)` | `Result[unit, string]` — online or offline per current state |
| `vm.restore(name: string)` | `Result[unit, string]` — resumes running iff taken online |
| `vm.snapshots()` | `Result[List[string], string]` |
| `vm.delete_snapshot(name: string)` | `Result[unit, string]` |

Share contents are outside snapshot scope.

## Related

- [Vm](../references/entity_vm_api.md)

[← Back to SKILL.md](../SKILL.md)
