# Networking & segments

_Virtual L2 segments with daemon DHCP/DNS, NAT, routing, port forwards and L3 filtering — all declarative._

Networking is **declarative** in `vmlab.wcl` — there is no `vmlab net` CLI. A
`segment` is a virtual L2 switch; the lab daemon supplies DHCP, DNS, NAT, routing
and L3 filtering in userspace. Runtime rule mutation is available from wscript via
the [Segment](../references/entity_seg_api.md) API.


```wcl
segment "name" {
  subnet    = "10.0.0.0/24"      // CIDR; auto-allocated from host pool if omitted
  global    = true               // owned by the supervisor, shared across labs (§9.2)
  dhcp      = true               // default true
  nat       = true               // internet egress for this segment (default false)
  routes_to = ["other_segment"]  // daemon inter-segment routing opt-in

  dns { server = "10.0.0.10" }   // hand out this server via DHCP instead of daemon DNS
  dns { enabled = false }        // or suppress DNS entirely

  connect { host = "helios:9999" }              // cross-host peer supervisor (PSK from host config)

  route   { dest = "10.60.0.0/24" via = "10.50.0.254" }   // pushed via DHCP option 121
  record  { name = "srv" ip = "10.0.0.5" }                // static DNS; wildcards OK ("*.internal")
  forward { host_port = 3389 to = "dc01:3389" proto = "tcp" }  // proto: "tcp" (default) | "udp" | "both"
  block   { cidr = "192.0.2.0/24" }                       // optional: proto = "tcp"|"udp"|"icmp", port = 443
  redirect { from = "10.0.0.1:80" to = "10.0.0.2:8080" }  // DNAT "ip[:port]"; optional proto
  sinkhole { pattern = "*.telemetry.com" mode = "nxdomain" }  // mode: "nxdomain" (default) | "zero"
}
```

`record` and `sinkhole` blocks may also appear at lab level (lab-wide). A `nic { nat = true }` shorthand attaches a VM to a per-lab built-in NAT segment without declaring one.

## Related

- [Labs](../references/concept_labs.md)

- [VM block](../references/concept_vms.md)

- [Daemon model](../references/concept_daemon_model.md)

[← All concepts](../references/concepts_ref.md)
