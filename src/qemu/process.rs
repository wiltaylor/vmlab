//! QEMU (and swtpm) process management: spawn with logs, watch for exit,
//! kill. Graceful-stop policy lives in the lab daemon's lifecycle (§7.2);
//! this layer is mechanics only.
//!
//! The waiter task owns the `Child` exclusively (holding a lock across
//! `child.wait()` would deadlock `kill()`); killing goes through a signal to
//! the recorded pid instead.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use anyhow::{Context, Result};
use tokio::process::Command;
use tokio::sync::watch;

/// A spawned VM (or helper) process.
pub struct Proc {
    pub name: String,
    /// 0 once the process has been reaped.
    pid: AtomicU32,
    /// Becomes Some(status_string) when the process exits.
    exited: watch::Receiver<Option<String>>,
}

impl Proc {
    /// Spawn `binary` with `args`, stdout+stderr appended to `log_path`.
    pub async fn spawn(
        name: &str,
        binary: &str,
        args: &[String],
        log_path: &Path,
    ) -> Result<Arc<Proc>> {
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let log = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .with_context(|| format!("opening {}", log_path.display()))?;
        let log_err = log.try_clone()?;

        let mut child = Command::new(binary)
            .args(args)
            .stdin(Stdio::null())
            .stdout(log)
            .stderr(log_err)
            .kill_on_drop(false)
            .spawn()
            .with_context(|| format!("spawning {binary} for {name}"))?;

        let (tx, rx) = watch::channel(None);
        let proc = Arc::new(Proc {
            name: name.to_string(),
            pid: AtomicU32::new(child.id().unwrap_or(0)),
            exited: rx,
        });

        // Waiter task: sole owner of the Child; reaps and publishes the
        // exit status.
        let watcher = proc.clone();
        tokio::spawn(async move {
            let status = child.wait().await;
            let s = match status {
                Ok(st) => st.to_string(),
                Err(e) => format!("wait failed: {e}"),
            };
            watcher.pid.store(0, Ordering::SeqCst);
            let _ = tx.send(Some(s));
        });

        Ok(proc)
    }

    pub fn pid(&self) -> Option<u32> {
        match self.pid.load(Ordering::SeqCst) {
            0 => None,
            p => Some(p),
        }
    }

    pub fn is_running(&self) -> bool {
        self.exited.borrow().is_none()
    }

    /// Exit status string, if the process has exited.
    pub fn exit_status(&self) -> Option<String> {
        self.exited.borrow().clone()
    }

    /// Wait for exit with a timeout. Ok(status) on exit, Err on timeout.
    pub async fn wait_exit(&self, timeout: std::time::Duration) -> Result<String> {
        let mut rx = self.exited.clone();
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            if let Some(s) = rx.borrow().clone() {
                return Ok(s);
            }
            tokio::time::timeout_at(deadline, rx.changed())
                .await
                .map_err(|_| anyhow::anyhow!("{} did not exit within {timeout:?}", self.name))?
                .map_err(|_| anyhow::anyhow!("process watcher gone"))?;
        }
    }

    /// SIGKILL the process (the hard end of the §7.2 stop ladder).
    pub async fn kill(&self) {
        if let Some(pid) = self.pid() {
            let _ = nix::sys::signal::kill(
                nix::unistd::Pid::from_raw(pid as i32),
                nix::sys::signal::Signal::SIGKILL,
            );
        }
    }

    /// SIGTERM (used for helpers like swtpm that exit cleanly on it).
    pub async fn terminate(&self) {
        if let Some(pid) = self.pid() {
            let _ = nix::sys::signal::kill(
                nix::unistd::Pid::from_raw(pid as i32),
                nix::sys::signal::Signal::SIGTERM,
            );
        }
    }
}

/// Spawn swtpm for a VM (PRD §5.3): TPM 2.0 emulator on a unix control
/// socket, state under `state_dir`.
pub async fn spawn_swtpm(
    vm_name: &str,
    state_dir: &Path,
    ctrl_sock: &Path,
    log_path: &Path,
) -> Result<Arc<Proc>> {
    std::fs::create_dir_all(state_dir)?;
    let args = vec![
        "socket".to_string(),
        "--tpm2".to_string(),
        "--tpmstate".to_string(),
        format!("dir={}", state_dir.display()),
        "--ctrl".to_string(),
        format!("type=unixio,path={}", ctrl_sock.display()),
        "--terminate".to_string(),
    ];
    Proc::spawn(&format!("swtpm:{vm_name}"), "swtpm", &args, log_path).await
}

/// Where the binaries live — verified at daemon start for clear errors.
pub fn check_binaries(arches: &[&str]) -> Vec<String> {
    let mut missing = Vec::new();
    for arch in arches {
        let bin = super::cmdline::emulator_binary(arch);
        if which(&bin).is_none() {
            missing.push(bin);
        }
    }
    for bin in ["qemu-img", "swtpm"] {
        if which(bin).is_none() {
            missing.push(bin.to_string());
        }
    }
    missing
}

fn which(bin: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|d| d.join(bin))
        .find(|p| p.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn spawn_watch_exit() {
        let tmp = tempfile::tempdir().unwrap();
        let log = tmp.path().join("p.log");
        let p = Proc::spawn("t", "sh", &["-c".into(), "echo hi; exit 3".into()], &log)
            .await
            .unwrap();
        let status = p
            .wait_exit(std::time::Duration::from_secs(5))
            .await
            .unwrap();
        assert!(status.contains('3'), "{status}");
        assert!(!p.is_running());
        assert!(p.pid().is_none());
        let logged = std::fs::read_to_string(&log).unwrap();
        assert_eq!(logged.trim(), "hi");
    }

    #[tokio::test]
    async fn kill_terminates_even_after_waiter_starts() {
        let tmp = tempfile::tempdir().unwrap();
        let p = Proc::spawn("t", "sleep", &["30".into()], &tmp.path().join("p.log"))
            .await
            .unwrap();
        assert!(p.is_running());
        // Let the waiter task start waiting first — this order used to
        // deadlock when the waiter held a lock across child.wait().
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        p.kill().await;
        let status = p
            .wait_exit(std::time::Duration::from_secs(5))
            .await
            .unwrap();
        assert!(status.contains("signal"), "{status}");
    }

    #[test]
    fn binaries_present_on_host() {
        assert!(check_binaries(&["x86_64"]).is_empty());
    }
}
