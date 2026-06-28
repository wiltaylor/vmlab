# Vm: lifecycle & state methods

| Method | Returns | Notes |
| --- | --- | --- |
| `vm.name()` | `string` |  |
| `vm.start()` / `vm.stop()` / `vm.stop_force()` / `vm.restart()` | `Result[unit, string]` | stop = graceful ladder (agent → ACPI → kill) |
| `vm.state()` | `string` | one of `"stopped"` / `"starting"` / `"running"` / `"stopping"` |
| `vm.is_ready()` | `bool` | Guest agent responding |
| `vm.wait_ready(timeout_secs: int)` | `Result[unit, string]` | Block until agent responds |
| `vm.wait_shutdown(timeout_secs: int)` | `Result[unit, string]` | Block until powered off |
| `vm.ip()` | `Result[string, string]` | Primary NIC IPv4 (DHCP lease / agent) |
| `vm.ip_nic(nic: int)` | `Result[string, string]` | By NIC index (0-based) |

## Related

- [Vm](../references/entity_vm_api.md)

- [Lab](../references/entity_lab_api.md)

[← Back to SKILL.md](../SKILL.md)
