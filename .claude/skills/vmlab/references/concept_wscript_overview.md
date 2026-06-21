# wscript: overview

_A statically typed, Rust-flavoured scripting language; vmlab type-checks scripts at `vmlab validate` time._

wscript is a statically typed, Rust-flavoured scripting language — think Rust minus
the borrow checker, lifetimes, and user generics. vmlab compiles scripts with full
type checking at `vmlab validate` time. Every script that drives a lab starts with
`use vmlab`. Scripts are synchronous; all blocking calls take timeouts and return
`Result[..., string]`. Generate `vmlab.wscripti` (`vmlab wscripti`) for LSP support
when editing scripts.


## Entry points

```rust
// Provision script (provision "x.wscript" {}) and `vmlab script x.wscript`:
fn main(lab: Lab) { ... }       // an Err propagating out fails the provision run (and `vmlab up`)

// Event handler (on "vm.crashed" { run = "x.wscript" }):
fn handle(event: Event, lab: Lab) { ... }   // failures logged, never fatal
```

## Examples

### A provision script driving a guest

fn main(lab: Lab): wait for the agent, run a command, then click a UI element by screen match.

```rust
use vmlab

fn main(lab: Lab) {
    lab.log("setting up " + lab.name())

    let Ok(dc) = lab.vm("dc01") else {
        lab.log("dc01 is not defined")
        return
    }

    match dc.wait_ready(600) {
        Ok(_)  => lab.log("dc01 agent is responding"),
        Err(e) => { lab.log("dc01 never became ready: " + e); return }
    }

    match dc.exec("ipconfig", ["/all"]) {
        Ok(r)  => lab.log(r.stdout),
        Err(e) => lab.log("ipconfig failed: " + e),
    }

    // Screen-driven step: wait for a UI element, click its center.
    match dc.wait_for_image("images/promote-button.png", 120) {
        Ok(m) => {
            let mv = dc.mouse_move(m.cx, m.cy)   // bind unused Results
            let cl = dc.mouse_click("left")
            lab.log("clicked the promote button")
        }
        Err(e) => lab.log("promote button not found (skipping): " + e),
    }
}
```

## Related

- [wscript: types & values](../references/concept_wscript_types.md)

- [wscript: pattern matching & errors](../references/concept_wscript_matching.md)

- [Lab](../references/entity_lab_api.md)

- [Provisions & event handlers](../references/concept_provisions.md)

[← All concepts](../references/concepts_ref.md)
