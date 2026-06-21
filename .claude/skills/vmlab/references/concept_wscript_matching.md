# wscript: pattern matching & errors

_Option\[T\] and Result\[T,E\] are built in; let-else, match (exhaustive), and ? are the idioms vmlab scripts live on._

`Option[T]` and `Result[T, E]` are built in. All fallible vmlab API calls return `Result[..., string]`.

```rust
// let-else: bail early (block must diverge — return/break)
let Ok(dc) = lab.vm("dc01") else {
    lab.log("dc01 is not defined")
    return
}

// match (exhaustiveness-checked at compile time)
match dc.wait_ready(600) {
    Ok(_)  => lab.log("ready"),
    Err(e) => { lab.log("not ready: " + e); return }
}

// ? propagates Err/None out of a function with a matching return type
fn step(lab: Lab) -> Result[unit, string] {
    let dc = lab.vm("dc01")?
    dc.wait_ready(600)?
    Ok(())
}
```

Methods: `is_some is_none unwrap unwrap_or expect` (Option) / `is_ok is_err unwrap unwrap_or unwrap_err expect` (Result).

## Related

- [wscript: overview](../references/concept_wscript_overview.md)

- [Lab](../references/entity_lab_api.md)

[← All concepts](../references/concepts_ref.md)
