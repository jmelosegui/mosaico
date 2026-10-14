//! The channel between the CLI and the daemon.
//!
//! A platform provides the byte streams; everything above them, the
//! framing, the one-request-one-reply exchange and the daemon's accept
//! loop, lives here so every platform behaves the same.

use std::io::{BufReader, Read, Write};
use std::time::Duration;

use serde_json::Value;

use super::framing::{read_message, write_message};
use super::{Command, ErrorCode, IpcError, Request, Response, decode_request};

/// How the CLI reaches the daemon.
///
/// # Contract for platform implementations
///
/// - A client is built for one daemon instance (an endpoint the platform
///   names, such as a pipe or socket path) and only ever reaches that one.
/// - `connect` fails with [`IpcError::NotRunning`] when no daemon is
///   listening, without waiting. When the daemon is listening but busy
///   with another client, it waits a short, bounded time for it to
///   become free before giving up.
/// - Once connected, reads wait until the daemon replies, goes away, or
///   the timeout set with [`IpcStream::set_read_timeout`] passes. A
///   daemon that exits mid-request must end the stream, so the client
///   sees [`IpcError::Closed`]; one that is alive but stuck is reported
///   as [`IpcError::TimedOut`]. Neither may leave the client hanging.
/// - `is_daemon_running` reports whether the daemon's endpoint exists. A
///   daemon busy answering another client is running. It must not
///   consume the daemon's attention: it checks without connecting.
/// - Callers about to send a command do not need `is_daemon_running`
///   first: `send` reports [`IpcError::NotRunning`] by itself, and a
///   separate check can only add a window in which the answer changes.
pub trait IpcClient {
    /// A connection to the daemon.
    type Stream: IpcStream;

    /// Whether a daemon is listening, busy or not.
    fn is_daemon_running(&self) -> bool;

    /// Opens a connection to the daemon.
    fn connect(&self) -> Result<Self::Stream, IpcError>;

    /// Sends one command and waits for its reply, for up to
    /// [`REPLY_TIMEOUT`].
    fn send(&self, command: &Command) -> Result<Response, IpcError> {
        self.send_with_timeout(command, REPLY_TIMEOUT)
    }

    /// Sends one command and waits for its reply, for up to `timeout`.
    fn send_with_timeout(
        &self,
        command: &Command,
        timeout: Duration,
    ) -> Result<Response, IpcError> {
        let mut stream = self.connect()?;
        write_message(&mut stream, &Request::new(command.clone()))?;
        // A daemon that is alive but stuck keeps the connection open and
        // never replies; without a limit the CLI would wait forever.
        stream.set_read_timeout(timeout)?;
        read_message(&mut BufReader::new(stream)).map_err(|e| match e {
            IpcError::Io(e) if e.kind() == std::io::ErrorKind::TimedOut => IpcError::TimedOut,
            other => other,
        })
    }
}

/// How long the CLI waits for the daemon's reply. Long enough for any
/// real command, including a retile of a crowded desktop, short enough
/// that a daemon that stopped answering is reported rather than waited
/// on forever.
pub const REPLY_TIMEOUT: Duration = Duration::from_secs(10);

/// The client's end of a connection to the daemon.
pub trait IpcStream: Read + Write {
    /// Limits how long reading the reply may take, counted from now.
    ///
    /// Once `timeout` has passed, reads fail with
    /// [`io::ErrorKind::TimedOut`](std::io::ErrorKind::TimedOut), which
    /// [`IpcClient::send`] reports as [`IpcError::TimedOut`]. A transport
    /// whose reads can block must implement it; the default, which does
    /// nothing, only suits streams that never block.
    fn set_read_timeout(&mut self, timeout: Duration) -> Result<(), IpcError> {
        let _ = timeout;
        Ok(())
    }
}

/// How the daemon listens for the CLI.
///
/// # Contract for platform implementations
///
/// - `accept` blocks until a client connects. Each connection carries
///   exactly one command and one reply.
/// - While one connection is being answered, the endpoint keeps
///   existing and accepts the next client, so a second CLI command run
///   at the same time never finds the daemon missing.
/// - A client that connected and went away before it could be handed
///   out is reported as [`IpcError::Closed`]. It concerns that client
///   only, and the server must still be listening afterwards; [`serve`]
///   waits for the next client.
/// - Any other error from `accept` means the daemon can no longer listen
///   at all; [`serve`] stops on it.
/// - Only clients of the same user may connect.
pub trait IpcServer {
    /// One accepted connection.
    type Connection: IpcConnection;

    /// Waits for the next client.
    fn accept(&mut self) -> Result<Self::Connection, IpcError>;
}

/// The daemon's end of one connection.
pub trait IpcConnection: Read + Write {
    /// Ends the connection once the reply is written.
    ///
    /// Must not return until the client can read the whole reply: on
    /// some transports, closing early throws away data the client has
    /// not read yet.
    fn finish(self) -> Result<(), IpcError>
    where
        Self: Sized;

    /// Limits how long reading the request may take, counted from now.
    ///
    /// Once `timeout` has passed, reads fail with
    /// [`io::ErrorKind::TimedOut`](std::io::ErrorKind::TimedOut), however
    /// little or much the client has sent. [`serve`] sets it on every
    /// connection, so a client that connects and never sends cannot hold
    /// up the clients after it. A transport whose reads can block must
    /// implement it; the default, which does nothing, only suits
    /// connections that never block.
    fn set_read_timeout(&mut self, timeout: Duration) -> Result<(), IpcError> {
        let _ = timeout;
        Ok(())
    }

    /// Limits how long writing the reply and [`finish`](Self::finish)
    /// may take, counted from now.
    ///
    /// Once `timeout` has passed, a write or `finish` still waiting for
    /// the client to read fails with
    /// [`io::ErrorKind::TimedOut`](std::io::ErrorKind::TimedOut) and the
    /// connection is dropped. [`serve`] sets it before every reply, so a
    /// client that never reads cannot hold up the clients after it. A
    /// transport whose writes or `finish` can wait for the client must
    /// implement it; the default, which does nothing, only suits
    /// connections that never wait.
    fn set_write_timeout(&mut self, timeout: Duration) -> Result<(), IpcError> {
        let _ = timeout;
        Ok(())
    }
}

/// Why [`serve`] returned.
#[derive(Debug)]
pub enum Served {
    /// The daemon was told to stop, and the client got its reply.
    Stopped,
    /// `handle` returned `None`: there is nobody left to answer.
    Abandoned,
    /// The server can no longer accept connections. The daemon has lost
    /// its only control channel and should shut down.
    Failed(IpcError),
}

/// Runs the daemon's side of the channel until it is told to stop.
///
/// For each connection, reads one request and answers it. Requests that
/// cannot be carried out (malformed, too large, unknown, from another
/// protocol version) get an error reply without reaching `handle`, so a
/// client is never left waiting. Everything else goes to `handle`.
pub fn serve<S: IpcServer>(
    server: &mut S,
    handle: impl FnMut(Command) -> Option<Response>,
) -> Served {
    serve_with_timeout(server, REQUEST_TIMEOUT, handle)
}

/// How long a client gets to send its request once connected, and again
/// to read the reply. A real CLI sends straight after connecting and
/// reads straight after sending, so this only ever runs out for a client
/// that is stuck or not a mosaico client at all.
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

/// [`serve`], with how long a client gets to send its request and to
/// read the reply.
pub fn serve_with_timeout<S: IpcServer>(
    server: &mut S,
    timeout: Duration,
    mut handle: impl FnMut(Command) -> Option<Response>,
) -> Served {
    loop {
        let mut connection = match server.accept() {
            Ok(connection) => connection,
            // One client went away before it could be served. The
            // server is still listening, so wait for the next one.
            Err(IpcError::Closed) => {
                crate::log_debug!("IPC: a client left before it was accepted");
                continue;
            }
            Err(e) => return Served::Failed(e),
        };

        // The connections are answered one at a time, so a client that
        // connects and never sends would hold up every client after it.
        if let Err(e) = connection.set_read_timeout(timeout) {
            crate::log_warn!("IPC: could not limit how long a request may take: {e}");
        }

        let request = read_message::<Value>(&mut BufReader::new(&mut connection))
            .map_err(|e| match e {
                IpcError::TooLarge => Some(Response::error(
                    ErrorCode::TooLarge,
                    "the request is too large",
                )),
                IpcError::Malformed(e) => Some(Response::error(
                    ErrorCode::Malformed,
                    format!("malformed request: {e}"),
                )),
                IpcError::Io(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                    crate::log_debug!("IPC: dropped a client that sent nothing within {timeout:?}");
                    None
                }
                // The client is gone; there is nobody to answer.
                _ => None,
            })
            .and_then(|value| decode_request(value).map_err(Some));

        let (response, is_stop) = match request {
            Ok(command) => {
                let is_stop = command == Command::Stop;
                match handle(command) {
                    Some(response) => (response, is_stop),
                    None => return Served::Abandoned,
                }
            }
            Err(Some(refusal)) => (refusal, false),
            Err(None) => continue,
        };

        // A client that sends its request and then never reads the reply
        // would otherwise hold up every client after it.
        if let Err(e) = connection.set_write_timeout(timeout) {
            crate::log_warn!("IPC: could not limit how long a reply may take: {e}");
        }
        let replied = write_message(&mut connection, &response).and_then(|()| connection.finish());
        if let Err(e) = replied {
            crate::log_warn!("IPC: could not reply: {e}");
        }
        if is_stop {
            return Served::Stopped;
        }
    }
}

#[cfg(test)]
#[path = "tests/transport_tests.rs"]
mod tests;
