//! Shared QMP data types.

use serde::Deserialize;
use serde_json::Value;

/// An asynchronous QMP event, e.g. `SHUTDOWN`, `STOP`, `RESET`.
#[derive(Debug, Clone)]
pub struct QmpEvent {
    /// Event name as emitted by QEMU (e.g. `"SHUTDOWN"`).
    pub event: String,
    /// Event payload (`data` member); `Value::Null` when absent.
    pub data: Value,
    /// Host-side timestamp QEMU attached to the event.
    pub timestamp: EventTimestamp,
}

/// Timestamp attached to QMP events (`{"seconds": ..., "microseconds": ...}`).
#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub struct EventTimestamp {
    #[serde(default)]
    pub seconds: i64,
    #[serde(default)]
    pub microseconds: i64,
}

/// VM run state as reported by `query-status`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunState {
    Running,
    Paused,
    Shutdown,
    Suspended,
    PreLaunch,
    InternalError,
    IoError,
    Watchdog,
    GuestPanicked,
    FinishMigrate,
    PostMigrate,
    RestoreVm,
    SaveVm,
    Debug,
    /// Any state this enum does not name explicitly; carries the raw
    /// `status` string from QEMU.
    Other(String),
}

impl RunState {
    /// Map a `query-status` `status` string onto a [`RunState`].
    pub fn from_status(status: &str) -> Self {
        match status {
            "running" => RunState::Running,
            "paused" => RunState::Paused,
            "shutdown" => RunState::Shutdown,
            "suspended" => RunState::Suspended,
            "prelaunch" => RunState::PreLaunch,
            "internal-error" => RunState::InternalError,
            "io-error" => RunState::IoError,
            "watchdog" => RunState::Watchdog,
            "guest-panicked" => RunState::GuestPanicked,
            "finish-migrate" => RunState::FinishMigrate,
            "postmigrate" => RunState::PostMigrate,
            "restore-vm" => RunState::RestoreVm,
            "save-vm" => RunState::SaveVm,
            "debug" => RunState::Debug,
            other => RunState::Other(other.to_string()),
        }
    }
}

/// A block node as returned by `query-named-block-nodes`.
///
/// Only the fields vmlab's snapshot routing needs are typed; everything
/// else QEMU reports is ignored on deserialisation.
#[derive(Debug, Clone, Deserialize)]
pub struct NamedBlockNode {
    /// Graph node name (used as `vmstate`/`devices` for snapshot commands).
    #[serde(rename = "node-name")]
    pub node_name: String,
    /// Block driver (e.g. `"qcow2"`, `"file"`).
    #[serde(default)]
    pub drv: Option<String>,
    /// Whether the node is read-only.
    #[serde(default)]
    pub ro: bool,
    /// Backing file path, if any.
    #[serde(default)]
    pub file: Option<String>,
}
