# Vm: guest agent methods

| Method | Returns | Notes |
| --- | --- | --- |
| `vm.exec(cmd: string, args: List[string])` | `Result[ExecResult, string]` | 120 s timeout |
| `vm.exec_timeout(cmd, args, timeout_secs: int)` | `Result[ExecResult, string]` | Custom timeout |
| `vm.copy_to(local: string, guest_path: string)` | `Result[unit, string]` | local relative to lab root; guest path absolute |
| `vm.copy_from(guest_path: string, local: string)` | `Result[unit, string]` | Parent dirs created on host |

Exec returns an [ExecResult](../references/entity_exec_result_type.md) (`exit_code`, `stdout`, `stderr`).

## Related

- [Vm](../references/entity_vm_api.md)

- [ExecResult](../references/entity_exec_result_type.md)

[← Back to SKILL.md](../SKILL.md)
