# Vm: keyboard & mouse methods

| Method | Returns | Notes |
| --- | --- | --- |
| `vm.send_keys(chord: string)` | `Result[unit, string]` | e.g. `"ctrl-alt-del"`, `"enter"`, `"shift-f5"` |
| `vm.type_text(text: string)` | `Result[unit, string]` | ~35 ms/key; `\n`→enter, `\t`→tab; **US ASCII only** |
| `vm.type_text_paced(text: string, delay_ms: int)` | `Result[unit, string]` | Custom inter-key delay |
| `vm.mouse_move(x: int, y: int)` | `Result[unit, string]` | Absolute, scaled to current screen size |
| `vm.mouse_click(button: string)` | `Result[unit, string]` | `"left"` / `"right"` / `"middle"` |
| `vm.mouse_drag(x1, y1, x2, y2: int)` | `Result[unit, string]` | Human-ish 8-step drag |

See [the chord-key reference](../references/fact_key_chords.md) for the full key-name vocabulary.

## Related

- [Vm](../references/entity_vm_api.md)

- [Keyboard chord names](../references/fact_key_chords.md)

[← Back to SKILL.md](../SKILL.md)
