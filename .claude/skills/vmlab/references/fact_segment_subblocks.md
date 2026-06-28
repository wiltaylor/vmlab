# segment {} sub-blocks

Blocks declared inside a `segment {}` (some also at lab level) configure DNS, routing, forwarding and L3 filtering:

```wcl
dns { server = "10.0.0.10" }   // hand out this server via DHCP instead of daemon DNS
dns { enabled = false }        // or suppress DNS entirely

connect { host = "helios:9999" }              // cross-host peer supervisor (PSK from host config)

route   { dest = "10.60.0.0/24" via = "10.50.0.254" }   // pushed via DHCP option 121
record  { name = "srv" ip = "10.0.0.5" }                // static DNS; wildcards OK ("*.internal")
forward { host_port = 3389 to = "dc01:3389" proto = "tcp" }  // proto: "tcp" (default) | "udp" | "both"
block   { cidr = "192.0.2.0/24" }                       // optional: proto = "tcp"|"udp"|"icmp", port = 443
redirect { from = "10.0.0.1:80" to = "10.0.0.2:8080" }  // DNAT "ip[:port]"; optional proto
sinkhole { pattern = "*.telemetry.com" mode = "nxdomain" }  // mode: "nxdomain" (default) | "zero"
```

`record` and `sinkhole` may also appear at lab level (lab-wide). Many of these rules can be mutated at runtime from wscript via [the Segment API](../references/entity_seg_api.md).

## Related

- [segment {} block](../references/entity_segment_block.md)

- [Networking model](../references/concept_networking.md)

- [Segment](../references/entity_seg_api.md)

[← Back to SKILL.md](../SKILL.md)
