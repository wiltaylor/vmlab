# wscript: functions & control flow

_Block-valued fn bodies, closures, if/for/while/loop; ranges are exclusive (0..10) or inclusive (0..=10)._

```rust
fn area(w: int, h: int) -> int { w * h }   // blocks evaluate to last expression
fn log_it(msg: string) { println(msg) }    // omitted return type = unit
let double = |x| x * 2                     // closures

if cond { } else { }
for i in 0..10 { }          // exclusive; 0..=10 inclusive
for x in [1, 2, 3] { }      // list elements (also map keys, string chars)
while cond { }
loop { if done { break } }
```

A function with **no return type must not end on a non-unit expression** — bind
unused Results (`let _ = vm.mouse_click("left")`) or add `;` to discard.


## Related

- [wscript: overview](../references/concept_wscript_overview.md)

[← All concepts](../references/concepts_ref.md)
