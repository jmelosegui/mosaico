use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// A reply from the daemon.
///
/// `code` and `data` are optional on the wire, so replies from daemons
/// that predate them still parse.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Response {
    /// Whether the command succeeded.
    pub status: ResponseStatus,
    /// Human-readable message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Why the command failed, for programs to act on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<ErrorCode>,
    /// Result of the command, when it has one. See [`Response::data`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Status of a daemon response.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseStatus {
    /// Command succeeded.
    Ok,
    /// Command failed.
    Error,
}

/// Machine-readable reason for a failed command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ErrorCode {
    /// The request was not a valid JSON object.
    Malformed,
    /// The request was larger than the daemon accepts.
    TooLarge,
    /// The daemon does not know this command.
    UnknownCommand,
    /// The request came from another protocol version. `data` holds the
    /// daemon's [`DaemonStatus`].
    VersionMismatch,
    /// The command was understood but could not be carried out.
    Failed,
    /// A code from a newer daemon that this build does not know.
    #[serde(other)]
    Other,
}

/// What a daemon reports about itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaemonStatus {
    /// The mosaico release the daemon was built from.
    pub version: String,
    /// The [`super::PROTOCOL_VERSION`] the daemon speaks.
    pub protocol: u32,
    /// Number of windows the daemon manages.
    #[serde(default)]
    pub windows: usize,
}

impl Response {
    /// Creates a successful response with no message.
    pub fn ok() -> Self {
        Self {
            status: ResponseStatus::Ok,
            message: None,
            code: None,
            data: None,
        }
    }

    /// Creates a successful response with a message.
    pub fn ok_with_message(message: impl Into<String>) -> Self {
        Self {
            message: Some(message.into()),
            ..Self::ok()
        }
    }

    /// Creates a failed response.
    pub fn error(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            status: ResponseStatus::Error,
            message: Some(message.into()),
            code: Some(code),
            data: None,
        }
    }

    /// Attaches a typed result to the response.
    ///
    /// A payload that cannot be represented as JSON is left out rather
    /// than failing the whole reply.
    pub fn with_data<T: Serialize>(mut self, data: &T) -> Self {
        self.data = serde_json::to_value(data).ok();
        self
    }

    /// Reads the typed result back, or `None` when there is none or it
    /// does not have the expected shape.
    pub fn data<T: DeserializeOwned>(&self) -> Option<T> {
        self.data
            .clone()
            .and_then(|value| serde_json::from_value(value).ok())
    }
}
