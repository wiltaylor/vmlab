# Guest OS profiles

_Shipped hardware-default sets (windows-11, linux-modern, …); override or extend by dropping \*.wcl into ~/.config/vmlab/profiles/._

A **profile** is a named set of hardware defaults (machine type, firmware, secure
boot, TPM, disk bus, NIC, display, CPUs/memory) chosen with `profile = "..."` on a
VM or template. Precedence is **VM block > template metadata > profile > defaults**.
Override or extend the shipped profiles by dropping `*.wcl` into
`~/.config/vmlab/profiles/`. See [the profiles table](../references/fact_profiles_table.md) for the
shipped set.


## Related

- [VM block](../references/concept_vms.md)

- [Templates](../references/concept_templates.md)

- [Host config](../references/concept_host_config.md)

[← All concepts](../references/concepts_ref.md)
