# Provisions & event handlers

_provision {} scripts run on `vmlab up`; on "event" {} handlers react to lifecycle events._

Provision scripts run on `vmlab up` in declaration order; event handlers react to lifecycle events. Both are wscript files.

```wcl
provision "scripts/setup.wscript" { }                     // runs on `vmlab up`, in order
provision "scripts/join.wscript"  { vms = ["client01"] }  // scoped: gates depends_on on these VMs

on "vm.crashed"    { run = "scripts/collect-dumps.wscript" }
on "host.disk_low" { run = "scripts/alert.wscript" }
```

**Provision failures fail `vmlab up`**; **handler failures are logged, never fatal.**
A scoped provision (`vms = [...]`) gates `depends_on` on those VMs: dependents wait
for the provision to finish. See [the event list](../references/fact_events.md) for every event name.


## Examples

### A crash event handler

fn handle(event, lab): screenshot the crashed VM. Failures here are logged, never fatal.

```rust
use vmlab

fn handle(event: Event, lab: Lab) {
    lab.log("crash handler fired for " + event.vm + " (" + event.name + ")")
    let Ok(vm) = lab.vm(event.vm) else { return }
    match vm.screenshot("") {
        Ok(path) => lab.log("saved crash screenshot: " + path),
        Err(e)   => lab.log("could not screenshot: " + e),
    }
}
```

## Related

- [Labs](../references/concept_labs.md)

- [wscript: pattern matching & errors](../references/concept_wscript_matching.md)

[← All concepts](../references/concepts_ref.md)
