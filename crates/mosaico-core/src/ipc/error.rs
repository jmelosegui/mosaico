use std::fmt;
use std::io;

/// Why an IPC exchange failed.
#[derive(Debug)]
pub enum IpcError {
    /// No daemon is listening.
    NotRunning,
    /// The other side went away before a whole message arrived.
    Closed,
    /// A message was larger than [`super::MAX_MESSAGE_BYTES`].
    TooLarge,
    /// A message was not valid JSON for the expected type.
    Malformed(serde_json::Error),
    /// The transport failed.
    Io(io::Error),
    /// The daemon is listening but did not reply in time.
    TimedOut,
}

impl fmt::Display for IpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotRunning => f.write_str("the daemon is not running"),
            Self::Closed => f.write_str("the connection closed before a reply arrived"),
            Self::TooLarge => f.write_str("message too large"),
            Self::Malformed(e) => write!(f, "malformed message: {e}"),
            Self::Io(e) => write!(f, "{e}"),
            Self::TimedOut => f.write_str("the daemon did not reply in time"),
        }
    }
}

impl std::error::Error for IpcError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Malformed(e) => Some(e),
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for IpcError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
