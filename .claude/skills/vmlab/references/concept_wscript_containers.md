# wscript: containers, strings & modules

_List and Map builtins, immutable strings, the always-on prelude, and `use vmlab`; scripts are single files in v1._

```rust
let xs = [1, 2, 3]                       // List[int]
xs.push(4); xs.get(99)                   // .get -> Option (never faults); xs[99] faults
xs.map(|x| x * 2).filter(|x| x > 2).fold(0, |a, x| a + x)
let ages = #{ "alice": 30 }              // Map[string, int]; keys: int/bool/char/string
```

**List:** \`len is_empty push pop get set insert remove clear contains index_of
reverse sort join map filter fold first last slice concat clone\`.
**Map:** `len is_empty insert remove get contains_key keys values clear clone`.
**String** (immutable): \`len bytes_len is_empty split trim trim_start trim_end
to_upper to_lower starts_with ends_with contains find replace repeat pad_left
pad_right chars slice parse_int parse_float\`.


`use vmlab` imports the vmlab host module (registered types like `Lab`, `Vm`,
`Match` are ambient — no `use` needed for type names). The always-available prelude:
`print println str fmt same weak int float`. Scripts are single files in v1 — no
script-to-script imports.


## Related

- [wscript: types & values](../references/concept_wscript_types.md)

- [wscript: overview](../references/concept_wscript_overview.md)

[← All concepts](../references/concepts_ref.md)
