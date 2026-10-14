//! The CLI to daemon protocol: messages, framing and transport.
//!
//! Every request is one JSON object holding the protocol version and the
//! command, for example `{"version":1,"command":"Stop"}`. Every reply is
//! a [`Response`]. Both travel as one line each, see [`framing`].

mod error;
pub mod framing;
mod request;
mod response;
mod transport;

use serde::{Deserialize, Serialize};

use crate::Action;

pub use error::IpcError;
pub use framing::{MAX_MESSAGE_BYTES, read_message, write_message};
pub use request::{Request, decode_request};
pub use response::{DaemonStatus, ErrorCode, Response, ResponseStatus};
pub use transport::{
    IpcClient, IpcConnection, IpcServer, IpcStream, REPLY_TIMEOUT, REQUEST_TIMEOUT, Served, serve,
    serve_with_timeout,
};

/// Version of the request and reply format.
///
/// Bump it whenever a change would make one side misread the other: a
/// new command, a renamed field, a different payload. The daemon refuses
/// requests from another version, except [`Command::Stop`], so a mismatched
/// daemon can always be stopped and restarted.
pub const PROTOCOL_VERSION: u32 = 1;

/// The mosaico release this build belongs to, reported by the daemon so
/// the CLI can name the version that needs restarting.
pub const DAEMON_VERSION: &str = env!("CARGO_PKG_VERSION");

/// A command sent from the CLI to the daemon.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "command")]
pub enum Command {
    /// Request the daemon to stop.
    Stop,
    /// Request the daemon's current status.
    Status,
    /// Execute a tiling action (focus, swap, retile, etc.).
    Action {
        /// The action to execute.
        action: Action,
    },
    /// Dump the daemon's internal workspace state for debugging.
    Inspect,
    /// Pause all hotkeys (unregister all except toggle-pause).
    PauseHotkeys,
    /// Re-register all hotkeys that were paused.
    UnpauseHotkeys,
}
