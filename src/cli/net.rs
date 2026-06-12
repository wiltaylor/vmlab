//! `vmlab net` — inspect and mutate network rules at runtime (PRD §9.9).

use anyhow::Result;
use serde_json::{Value, json};

use super::lab::current_lab;
use crate::cli::daemon;

#[derive(clap::Subcommand)]
pub enum NetCmd {
    /// List L3 rules across the lab's segments
    Rules,
    /// Block traffic to a CIDR/IP on a segment
    Block { segment: String, cidr: String },
    /// DNAT-redirect ip[:port] to ip[:port] on a segment
    Redirect {
        segment: String,
        from: String,
        to: String,
    },
    /// Forward a host port to a guest port
    Forward {
        segment: String,
        host_port: u16,
        vm: String,
        guest_port: u16,
    },
}

pub fn cmd_net(cmd: NetCmd) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let (name, _root) = current_lab()?;
        let client = daemon::try_lab_daemon(&name)
            .await
            .ok_or_else(|| anyhow::anyhow!("lab \"{name}\" is not running"))?;
        let remote = |e: crate::proto::ProtoError| anyhow::anyhow!("{e}");
        match cmd {
            NetCmd::Rules => {
                let v = client.call("net.rules", Value::Null).await.map_err(remote)?;
                print_rules(&v);
            }
            NetCmd::Block { segment, cidr } => {
                let v = client
                    .call("net.block", json!({"segment": segment, "cidr": cidr}))
                    .await
                    .map_err(remote)?;
                println!("blocked {cidr} on {segment} (rule {})", v["id"]);
            }
            NetCmd::Redirect { segment, from, to } => {
                let v = client
                    .call("net.redirect", json!({"segment": segment, "from": from, "to": to}))
                    .await
                    .map_err(remote)?;
                println!("redirect {from} -> {to} on {segment} (rule {})", v["id"]);
            }
            NetCmd::Forward { segment, host_port, vm, guest_port } => {
                let v = client
                    .call(
                        "net.forward",
                        json!({"segment": segment, "host_port": host_port, "vm": vm, "guest_port": guest_port}),
                    )
                    .await
                    .map_err(remote)?;
                println!("forward host:{host_port} -> {vm}:{guest_port} (rule {})", v["id"]);
            }
        }
        Ok(())
    })
}

fn print_rules(v: &Value) {
    let segs = v.as_array().cloned().unwrap_or_default();
    for seg in segs {
        let rules = seg["rules"].as_array().cloned().unwrap_or_default();
        if rules.is_empty() {
            continue;
        }
        println!("segment \"{}\"", seg["segment"].as_str().unwrap_or("?"));
        for r in rules {
            println!(
                "  [{}] {}",
                r["id"].as_u64().unwrap_or(0),
                r["description"].as_str().unwrap_or("?")
            );
        }
    }
}
