# What `vmlab validate` checks

`vmlab validate` performs schema + semantic checks with no side effects. It verifies:

| Check | Detail |
| --- | --- |
| WCL schema | The file conforms to the vmlab schema |
| Template refs | Exist in the store, or a registry ref is given with explicit `arch` |
| NIC segments | Every NIC's segment is declared |
| Static IPs | Inside the declared subnet; no duplicate static IPs or MACs |
| Dependencies | No `depends_on` cycles |
| Scripts | Provision/handler files exist AND compile (full wscript type-check) |
| Scratch VMs | Have `arch` + `profile` + `disk` |

## Related

- [Labs](../references/concept_labs.md)

- [VM block](../references/concept_vms.md)

- [Provisions & event handlers](../references/concept_provisions.md)

[← All facts](../references/facts_ref.md)
