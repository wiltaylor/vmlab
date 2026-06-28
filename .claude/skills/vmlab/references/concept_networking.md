# Networking model

_Networking is declarative: virtual L2 segments with daemon-supplied DHCP/DNS/NAT/routing/L3 filtering in userspace — no `vmlab net` CLI._

Networking is **declarative** in `vmlab.wcl` — there is no `vmlab net` CLI. A
`segment` is a virtual L2 switch; the lab daemon supplies DHCP, DNS, NAT, routing
and L3 filtering entirely in userspace (which is why vmlab needs no `CAP_NET_ADMIN`,
tap or bridge). Runtime rule mutation is available from wscript via the
[Segment](../references/entity_seg_api.md) API.


Segments are declared with `segment {}` blocks (see [the segment block](../references/entity_segment_block.md)
and [its sub-blocks](../references/fact_segment_subblocks.md)). `record` and `sinkhole` blocks may
also appear at lab level (lab-wide). A `nic { nat = true }` shorthand attaches a VM
to a per-lab built-in NAT segment without declaring one.


## Related

- [lab {} block](../references/entity_labs.md)

- [vm {} block](../references/entity_vms.md)

- [segment {} block](../references/entity_segment_block.md)

- [segment {} sub-blocks](../references/fact_segment_subblocks.md)

- [Daemon model](../references/concept_daemon_model.md)

- [Segment](../references/entity_seg_api.md)

- [The vmlab.wcl schema](../references/fact_schema_reference.md)

[← Back to SKILL.md](../SKILL.md)
