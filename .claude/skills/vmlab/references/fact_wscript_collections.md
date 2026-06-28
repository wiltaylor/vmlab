# wscript: List & Map methods

```rust
let xs = [1, 2, 3]                       // List[int]
xs.push(4); xs.get(99)                   // .get -> Option (never faults); xs[99] faults
xs.map(|x| x * 2).filter(|x| x > 2).fold(0, |a, x| a + x)
let ages = #{ "alice": 30 }              // Map[string, int]; keys: int/bool/char/string
```

**List:** \`len is_empty push pop get set insert remove clear contains index_of
reverse sort join map filter fold first last slice concat clone\`.
**Map:** `len is_empty insert remove get contains_key keys values clear clone`.


## Related

- [wscript: types & values](../references/concept_wscript_types.md)

- [wscript: modules & prelude](../references/concept_wscript_modules.md)

[← Back to SKILL.md](../SKILL.md)
