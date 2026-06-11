//! The per-lab daemon (PRD §3): owns the lab's QEMU processes, QMP/agent
//! channels, lab-local segments and network services, snapshots, state, and
//! events. One process per running lab, spawned and reaped by the
//! supervisor; the CLI talks to it directly for lab-scoped operations.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use serde_json::{Value, json};

use crate::config::LabFile;
use crate::proto::server::{Handler, Server, Streamer};

pub struct LabDaemon {
    pub name: String,
    pub root: PathBuf,
    pub config: tokio::sync::RwLock<LabFile>,
    pub events: tokio::sync::OnceCell<tokio::sync::broadcast::Sender<crate::proto::Event>>,
}

/// Entry point for `vmlab __labd --lab <name> --root <dir>`.
pub fn run(lab: String, root: PathBuf) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(run_async(lab, root))
}

async fn run_async(lab: String, root: PathBuf) -> Result<()> {
    let config = crate::config::load_lab_root(&root)
        .map_err(|e| anyhow::anyhow!("cannot load lab config: {e}"))?;
    anyhow::ensure!(
        config.lab.name == lab,
        "lab file at {} defines \"{}\", not \"{lab}\"",
        root.display(),
        config.lab.name
    );

    let daemon = Arc::new(LabDaemon {
        name: lab.clone(),
        root,
        config: tokio::sync::RwLock::new(config),
        events: tokio::sync::OnceCell::new(),
    });

    let sock = crate::paths::lab_socket(&lab);
    let handler: Arc<dyn Handler> = Arc::new(LabdHandler(daemon.clone()));
    let server = Server::bind(&sock, handler)
        .await
        .with_context(|| format!("binding {}", sock.display()))?;
    let _ = daemon.events.set(server.events.clone());

    tracing::info!("lab daemon for {lab} listening on {}", sock.display());
    futures::future::pending::<()>().await;
    drop(server);
    Ok(())
}

impl LabDaemon {
    pub fn emit(&self, event: crate::proto::Event) {
        if let Some(tx) = self.events.get() {
            let _ = tx.send(event);
        }
    }
}

struct LabdHandler(Arc<LabDaemon>);

#[async_trait::async_trait]
impl Handler for LabdHandler {
    async fn handle(&self, cmd: &str, _args: Value, _stream: &Streamer) -> Result<Value, String> {
        let d = &self.0;
        match cmd {
            "ping" => Ok(json!("pong")),
            "status" => {
                let config = d.config.read().await;
                Ok(json!({
                    "lab": d.name,
                    "root": d.root,
                    "vms": config.lab.vms.iter().map(|v| &v.name).collect::<Vec<_>>(),
                }))
            }
            "shutdown" => {
                tracing::info!("lab daemon shutdown requested");
                tokio::spawn(async {
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                    std::process::exit(0);
                });
                Ok(json!(true))
            }
            _ => Err(format!("unknown command `{cmd}`")),
        }
    }
}
