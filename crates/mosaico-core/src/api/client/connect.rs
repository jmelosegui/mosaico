//! How a program reaches the daemon's bar API.

use std::io::{Read, Write};

use crate::ipc::IpcError;

/// One connection to the daemon's bar API, in two halves that may be used
/// from different threads.
pub struct ClientConnection {
    /// What the daemon sends.
    pub reader: Box<dyn Read + Send>,
    /// What the daemon is sent.
    pub writer: Box<dyn Write + Send>,
}

/// How a program reaches the daemon's bar API.
///
/// # Contract for platform implementations
///
/// - `connect` opens a connection to the bar API endpoint of the current
///   instance's daemon, the one its `ApiListener` listens on, apart from
///   the CLI's.
/// - It fails with [`IpcError::NotRunning`] when no daemon listens, with
///   [`IpcError::TimedOut`] when one listens but takes no connection
///   within a few seconds, and with [`IpcError::Untrusted`] when the
///   endpoint is not the daemon's: held by another user, or by a program
///   less trusted than the user's own.
/// - The endpoint may learn who the client is, and nothing more: it cannot
///   act as the client.
/// - The two halves never wait for each other: a read waiting for the
///   daemon does not hold up a write.
/// - Reading gives what the daemon wrote before it closed its end, then
///   the end (no bytes).
pub trait ApiClient {
    /// Opens one connection.
    fn connect(&self) -> Result<ClientConnection, IpcError>;
}
