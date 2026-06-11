//! Per-lab network assembly (PRD §9): one switch per segment, subnet
//! allocation from the host pool, NIC listener sockets for QEMU stream
//! netdevs. Gateway services (DHCP/DNS/NAT) attach per segment.

use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result, bail};
use ipnet::Ipv4Net;

use crate::config::model::{Lab, Segment};
use crate::net::switch::{PortClass, Switch};

/// Name of the built-in per-lab NAT segment (`nic { nat = true }`, §9.7).
pub const NAT_SEGMENT: &str = "nat";

/// Default auto-allocation pool (PRD §9.4); /24s carved out of it.
pub const DEFAULT_POOL: &str = "10.213.0.0/16";

pub struct SegmentNet {
    pub name: String,
    pub switch: Arc<Switch>,
    pub subnet: Ipv4Net,
    pub gateway_ip: Ipv4Addr,
    /// Declared config (None for the built-in NAT segment).
    pub config: Option<Segment>,
    /// NAT egress on (declared `nat = true`, or the built-in segment).
    pub nat: bool,
    pub dhcp: bool,
    listeners: Vec<tokio::task::JoinHandle<()>>,
}

impl SegmentNet {
    /// Listen on a unix socket for one VM NIC; QEMU connects to it.
    pub async fn listen_nic(&mut self, sock: &PathBuf, isolated: bool) -> Result<()> {
        let handle = self
            .switch
            .listen_unix(sock, PortClass::Guest { isolated })
            .await
            .with_context(|| format!("listening on {}", sock.display()))?;
        self.listeners.push(handle);
        Ok(())
    }
}

pub struct LabNetwork {
    pub segments: HashMap<String, SegmentNet>,
}

impl LabNetwork {
    /// Build switches and allocate subnets for every declared segment, plus
    /// the built-in NAT segment when any NIC uses `nat = true`.
    pub fn build(lab: &Lab) -> Result<LabNetwork> {
        let pool: Ipv4Net = DEFAULT_POOL.parse().expect("valid pool");
        let declared: Vec<Ipv4Net> = lab.segments.iter().filter_map(|s| s.subnet).collect();

        let mut auto = pool
            .subnets(24)
            .expect("pool splits into /24s")
            .filter(|c| {
                !declared
                    .iter()
                    .any(|d| d.contains(&c.network()) || c.contains(&d.network()))
            });
        let mut alloc_auto = || -> Result<Ipv4Net> {
            auto.next()
                .ok_or_else(|| anyhow::anyhow!("auto-subnet pool exhausted"))
        };

        let mut segments = HashMap::new();
        for seg in &lab.segments {
            let subnet = match seg.subnet {
                Some(s) => s,
                None => alloc_auto()?,
            };
            segments.insert(
                seg.name.clone(),
                SegmentNet {
                    name: seg.name.clone(),
                    switch: Switch::new(format!("{}/{}", lab.name, seg.name)),
                    subnet,
                    gateway_ip: crate::config::validate::gateway_ip(subnet),
                    config: Some(seg.clone()),
                    nat: seg.nat,
                    dhcp: seg.dhcp,
                    listeners: Vec::new(),
                },
            );
        }

        let needs_nat_segment = lab.vms.iter().flat_map(|v| &v.nics).any(|n| n.nat);
        if needs_nat_segment {
            if segments.contains_key(NAT_SEGMENT) {
                bail!(
                    "a declared segment is named \"{NAT_SEGMENT}\" while `nic {{ nat = true }}` is \
                     also used — rename the segment (the name is reserved for the built-in NAT \
                     segment, PRD §9.7)"
                );
            }
            let subnet = alloc_auto()?;
            segments.insert(
                NAT_SEGMENT.to_string(),
                SegmentNet {
                    name: NAT_SEGMENT.to_string(),
                    switch: Switch::new(format!("{}/{}", lab.name, NAT_SEGMENT)),
                    subnet,
                    gateway_ip: crate::config::validate::gateway_ip(subnet),
                    config: None,
                    nat: true,
                    dhcp: true,
                    listeners: Vec::new(),
                },
            );
        }

        Ok(LabNetwork { segments })
    }

    pub fn segment_mut(&mut self, name: &str) -> Option<&mut SegmentNet> {
        self.segments.get_mut(name)
    }
}

/// Segment a NIC attaches to: its declared segment, or the built-in NAT
/// segment for `nat = true`.
pub fn nic_segment_name(nic: &crate::config::model::Nic) -> &str {
    if nic.nat {
        NAT_SEGMENT
    } else {
        nic.segment.as_deref().expect("validated nic")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::load_lab_source;
    use std::path::Path;

    fn lab(src: &str) -> Lab {
        load_lab_source(src, "<t>", Path::new("/tmp")).unwrap().lab
    }

    #[test]
    fn declared_and_auto_subnets() {
        let l = lab(r#"import <vmlab.wcl>
lab "l" {
  segment "corp" { subnet = "10.50.0.0/24" }
  segment "dmz" { }
  vm "a" { template = "x86_64/t" nic { nat = true } }
}"#);
        let net = LabNetwork::build(&l).unwrap();
        assert_eq!(net.segments["corp"].subnet.to_string(), "10.50.0.0/24");
        assert_eq!(net.segments["corp"].gateway_ip.to_string(), "10.50.0.1");
        // dmz auto-allocated from the pool.
        let dmz = net.segments["dmz"].subnet;
        assert!(dmz.to_string().starts_with("10.213."));
        // Built-in NAT segment exists and got its own /24.
        let nat = &net.segments[NAT_SEGMENT];
        assert!(nat.nat);
        assert_ne!(nat.subnet, dmz);
    }

    #[test]
    fn declared_subnet_inside_pool_not_reallocated() {
        let l = lab(r#"import <vmlab.wcl>
lab "l" {
  segment "a" { subnet = "10.213.0.0/24" }
  segment "b" { }
  vm "x" { template = "x86_64/t" nic { segment = "a" } }
}"#);
        let net = LabNetwork::build(&l).unwrap();
        assert_eq!(net.segments["a"].subnet.to_string(), "10.213.0.0/24");
        assert_ne!(net.segments["b"].subnet.to_string(), "10.213.0.0/24");
    }

    #[test]
    fn reserved_nat_name_conflict() {
        let l = lab(r#"import <vmlab.wcl>
lab "l" {
  segment "nat" { }
  vm "a" { template = "x86_64/t" nic { nat = true } }
}"#);
        assert!(LabNetwork::build(&l).is_err());
    }
}
