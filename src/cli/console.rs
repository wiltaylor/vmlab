//! `vmlab console` — attach a viewer to a VM's VNC display (PRD §11).
//!
//! Every VM serves VNC on a unix socket. We either launch a configured
//! viewer against it, or — for environments where the viewer lives
//! elsewhere (WSL2's Windows side) — bridge the unix socket to a localhost
//! TCP port and print the address (PRD §11, §13).

use std::path::Path;

use anyhow::{Context, Result, anyhow, bail};

use super::lab::{current_lab, split_vm_ref};
use crate::cli::daemon;

pub fn cmd_console(vm_ref: &str, tcp: bool) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let (lab_opt, vm) = split_vm_ref(vm_ref)?;
        let lab = match lab_opt {
            Some(l) => l,
            None => current_lab()?.0,
        };
        // Ensure the lab daemon is up so the socket exists.
        let _client = daemon::try_lab_daemon(&lab)
            .await
            .ok_or_else(|| anyhow!("lab \"{lab}\" is not running — `vmlab up` first"))?;

        let sock = crate::paths::lab_runtime_dir(&lab)
            .join("vms")
            .join(&vm)
            .join("vnc.sock");
        if !sock.exists() {
            bail!("VNC socket for {lab}/{vm} not found ({})", sock.display());
        }

        let host_cfg = crate::config::host::HostConfig::load_default().unwrap_or_default();

        if tcp || host_cfg.viewer.is_none() {
            // Bridge unix → TCP on a free localhost port and print it; the
            // user points any VNC client at it (the WSL2 path: a Windows
            // viewer over WSL localhost forwarding).
            let port = bridge_unix_to_tcp(&sock).await?;
            println!("VNC for {lab}/{vm} on 127.0.0.1:{port}");
            println!("connect a VNC viewer there (Ctrl-C to stop the bridge)");
            // Hold the bridge open until interrupted.
            tokio::signal::ctrl_c().await.ok();
            Ok(())
        } else {
            let viewer = host_cfg.viewer.unwrap();
            launch_viewer(&viewer, &sock)
        }
    })
}

/// Spawn a TCP listener that proxies one (or more) connections to the VM's
/// VNC unix socket. Returns the bound port.
async fn bridge_unix_to_tcp(sock: &Path) -> Result<u16> {
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
        .await
        .context("binding console TCP bridge")?;
    let port = listener.local_addr()?.port();
    let sock = sock.to_path_buf();
    tokio::spawn(async move {
        loop {
            let Ok((tcp, _)) = listener.accept().await else {
                break;
            };
            let sock = sock.clone();
            tokio::spawn(async move {
                if let Ok(unix) = tokio::net::UnixStream::connect(&sock).await {
                    let (mut tr, mut tw) = tcp.into_split();
                    let (mut ur, mut uw) = unix.into_split();
                    let a = tokio::io::copy(&mut tr, &mut uw);
                    let b = tokio::io::copy(&mut ur, &mut tw);
                    let _ = tokio::join!(a, b);
                }
            });
        }
    });
    Ok(port)
}

/// Launch a viewer command. `{}` in the template is replaced by the socket
/// path; otherwise the socket is appended as `unix://<path>`.
fn launch_viewer(viewer: &str, sock: &Path) -> Result<()> {
    let target = sock.display().to_string();
    let cmd = if viewer.contains("{}") {
        viewer.replace("{}", &target)
    } else {
        format!("{viewer} unix://{target}")
    };
    let mut parts = cmd.split_whitespace();
    let prog = parts
        .next()
        .ok_or_else(|| anyhow!("empty viewer command"))?;
    std::process::Command::new(prog)
        .args(parts)
        .spawn()
        .with_context(|| format!("launching viewer `{viewer}`"))?;
    println!("launched {viewer}");
    Ok(())
}
