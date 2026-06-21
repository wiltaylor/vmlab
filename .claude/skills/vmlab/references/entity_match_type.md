# Match / ExecResult / Event

_wscript data type_

The struct return types: Match (image/OCR hit), ExecResult (guest exec), Event (handler payload).

```rust
struct Match     { x: int, y: int, w: int, h: int, score: float,
                   cx: int, cy: int,      // center — feed to mouse_move
                   text: string }         // set by wait_for_text only
struct ExecResult { exit_code: int, stdout: string, stderr: string }
struct Event      { name: string, vm: string, data: string }   // data = JSON payload as text
```

## Related

- [Vm](../references/entity_vm_api.md)

- [Lifecycle events](../references/fact_events.md)

[← All entities](../references/entities_ref.md)
