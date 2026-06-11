//! The userspace network fabric (PRD §9).
//!
//! - [`frame`]: ethernet/ARP/IPv4/UDP/TCP/ICMP views and builders.
//! - [`framing`]: the 4-byte big-endian length framing QEMU uses for
//!   stream-socket netdevs.
//! - [`switch`]: the per-segment MAC-learning L2 switch with port isolation
//!   and the ingress-hook seam for L3 rules.

pub mod dhcp;
pub mod dns;
pub mod frame;
pub mod framing;
pub mod gateway;
pub mod nat;
pub mod rules;
pub mod switch;

// Re-exports for consumers (DHCP/DNS/NAT/daemon modules); nothing inside the
// bin uses them yet, hence the allow.
#[allow(unused_imports)]
pub use frame::{
    ArpOp, ArpView, EthView, IcmpView, Ipv4View, TcpFields, TcpView, UdpView, arp_reply_build,
    arp_request_build, eth_build, icmp_build, icmp_echo_reply_for, icmp_unreachable_for,
    internet_checksum, ipv4_build, l4_checksum, tcp_build, udp_build,
};
#[allow(unused_imports)]
pub use framing::{MAX_FRAME_LEN, read_frame, write_frame};
#[allow(unused_imports)]
pub use switch::{ChannelPort, HookAction, IngressHook, PortClass, PortId, Switch, SwitchStats};
