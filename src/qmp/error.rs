//! QMP error type.

use thiserror::Error;

/// Errors produced by the QMP client.
#[derive(Debug, Error)]
pub enum QmpError {
    /// Underlying socket I/O failed.
    #[error("qmp i/o error: {0}")]
    Io(#[from] std::io::Error),

    /// A message could not be serialised or deserialised.
    #[error("qmp json error: {0}")]
    Json(#[from] serde_json::Error),

    /// The peer violated the QMP protocol (bad greeting, missing fields).
    #[error("qmp protocol error: {0}")]
    Protocol(String),

    /// QEMU rejected a command with `{"error": {"class", "desc"}}`.
    #[error("qmp command failed: {class}: {desc}")]
    Command { class: String, desc: String },

    /// A QMP background job (snapshot-save and friends) concluded with an
    /// error.
    #[error("qmp job '{job_id}' failed: {error}")]
    JobFailed { job_id: String, error: String },

    /// The connection is closed (EOF, `close()`, or reader task gone).
    #[error("qmp connection closed")]
    Closed,
}

impl QmpError {
    /// Build a [`QmpError::Command`] from a QMP `error` object.
    pub(crate) fn from_error_object(error: &serde_json::Value) -> Self {
        let class = error
            .get("class")
            .and_then(|c| c.as_str())
            .unwrap_or("GenericError")
            .to_string();
        let desc = error
            .get("desc")
            .and_then(|d| d.as_str())
            .unwrap_or_default()
            .to_string();
        QmpError::Command { class, desc }
    }
}
