# Lifecycle events

Events fire `on "<event>" {}` handlers and arrive in `fn handle(event: Event, lab: Lab)`. Handler failures are logged, never fatal.

| Event | Meaning |
| --- | --- |
| `vm.starting` | A VM has begun booting |
| `vm.ready` | The guest agent is responding |
| `vm.stopped` | A VM powered off cleanly |
| `vm.crashed` | A VM died unexpectedly (includes closing a `gui` window) |
| `lab.up` | The lab finished coming up |
| `lab.down` | The lab stopped |
| `snapshot.created` | A snapshot was taken |
| `snapshot.restored` | A snapshot was restored |
| `template.built` | A template build sealed into the store |
| `lab.daemon_crashed` | A lab daemon died (no auto-restart) |
| `host.disk_low` | Free disk fell below `disk_low_percent` |

## Related

- [Provisions & event handlers](../references/concept_provisions.md)

[← All facts](../references/facts_ref.md)
