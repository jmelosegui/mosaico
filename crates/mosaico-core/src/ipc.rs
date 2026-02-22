use serde::{Deserialize, Serialize};

use crate::Action;

/// The named pipe path used for IPC between CLI and daemon.
pub const PIPE_NAME: &str = r"\\.\pipe\mosaico";

/// A command sent from the CLI to the daemon.
///
/// These are serialized as JSON and sent over the named pipe.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "command")]
pub enum Command {
    /// Request the daemon to stop.
    Stop,
    /// Request the daemon's current status.
    Status,
    /// Execute a tiling action (focus, swap, retile, etc.).
    Action { action: Action },
}

/// A response sent from the daemon back to the CLI.
#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    /// Whether the command succeeded.
    pub status: ResponseStatus,
    /// Optional human-readable message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Status of a daemon response.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ResponseStatus {
    Ok,
    Error,
}

impl Response {
    /// Creates a successful response with no message.
    pub fn ok() -> Self {
        Self {
            status: ResponseStatus::Ok,
            message: None,
        }
    }

    /// Creates a successful response with a message.
    pub fn ok_with_message(message: impl Into<String>) -> Self {
        Self {
            status: ResponseStatus::Ok,
            message: Some(message.into()),
        }
    }
}
