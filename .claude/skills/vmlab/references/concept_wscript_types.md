# wscript: types & values

_64-bit ints, floats, strings, bool; no implicit numeric conversion, no truthiness, reference semantics for compound types._

```rust
let x = 5                  // int (64-bit signed, wrapping); inference everywhere
let name: string = "wil"   // annotations allowed on lets, required only on fn signatures
let pi = 3.14              // float
let log = "hp: " + str(99)         // + concatenates strings; str() converts
let msg = fmt("{} of {}", 3, 10)   // {} formatting (no string interpolation)
```

- **No implicit numeric conversion**: `1 + 2.0` is a type error — use `int(x)` / `float(x)`.
- **No truthiness**: conditions must be `bool`.
- Statements end at newlines; semicolons optional. Lines starting with `.` continue a method chain.
- Reference semantics for `string`/structs/lists/maps (assignment copies the reference); `clone()` for deep copies.


## Related

- [wscript: overview](../references/concept_wscript_overview.md)

- [wscript: containers, strings & modules](../references/concept_wscript_containers.md)

[← All concepts](../references/concepts_ref.md)
