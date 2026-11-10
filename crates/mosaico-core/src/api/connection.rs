//! The channel between the daemon and the bars, as a platform provides it.
//!
//! A platform provides the endpoint and the byte streams; the sessions,
//! their messages and their limits live in this module, so every platform
//! behaves the same.

use std::io::{self, Read, Write};
use std::time::Duration;

use crate::ipc::IpcError;

/// Stops a listener from any thread.
pub type Stopper = Box<dyn Fn() + Send + Sync>;

/// How the daemon listens for bars.
///
/// # Contract for platform implementations
///
/// - The endpoint is the instance's own and apart from the CLI's: a
///   session lasts as long as its bar runs and is spoken by a program of
///   another release.
/// - `accept` blocks until a bar connects. Any number of connections may
///   be open at once, each as long as its bar likes, and the endpoint
///   keeps accepting while they are.
/// - Once the [`Stopper`] from [`stopper`](Self::stopper) is called, from
///   any thread, `accept` returns `Ok(None)` at once, and from then on.
/// - A client that connected and went away before it could be handed out
///   is reported as [`IpcError::Closed`]; the listener keeps listening.
///   Any other error means it can no longer listen.
/// - Only processes of the same user, on the same machine, may connect,
///   and none less trusted than the user's ordinary programs. Unlike the
///   CLI's endpoint, an ordinary program may connect to a daemon that
///   runs with more rights: a bar runs as the user does.
/// - If another program already holds the endpoint's name, creating it
///   fails rather than sharing it, and `accept` reports the error.
pub trait ApiListener {
    /// One accepted connection.
    type Connection: ApiConnection;

    /// Waits for the next bar, or `None` once stopped.
    fn accept(&mut self) -> Result<Option<Self::Connection>, IpcError>;

    /// What stops this listener.
    fn stopper(&self) -> Stopper;
}

/// The daemon's end of a connection to one bar.
pub trait ApiConnection: Send + 'static {
    /// The half the bar's messages are read from.
    type Reader: ApiReader;
    /// The half the daemon's messages are written to.
    type Writer: ApiWriter;

    /// Splits the connection, so one thread reads while another writes.
    /// Neither half waits for the other.
    fn split(self) -> Result<(Self::Reader, Self::Writer), IpcError>;
}

/// Reads what a bar sends.
pub trait ApiReader: Read + Send + 'static {
    /// Limits how long reading may take from now on, or lifts the limit
    /// with `None`. Once it has passed, reads fail with
    /// [`io::ErrorKind::TimedOut`], however little or much came.
    fn set_read_timeout(&mut self, timeout: Option<Duration>) -> io::Result<()>;
}

/// Writes to a bar.
pub trait ApiWriter: Write + Send + 'static {
    /// Limits how long writing may take from now on. Once it has passed, a
    /// write still waiting for the bar to read fails with
    /// [`io::ErrorKind::TimedOut`].
    fn set_write_timeout(&mut self, timeout: Duration) -> io::Result<()>;

    /// Ends the connection. A read waiting on the other half returns at
    /// once, and so do reads started later. What was written stays for the
    /// bar to read, then it sees the end.
    fn shutdown(&mut self);
}

/// How far sessions may go before the daemon stops serving them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// How long a bar has, once connected, to send `hello`.
    pub hello: Duration,
    /// How long a message may take to write before the session is dropped.
    pub write: Duration,
    /// How many sessions may be open, past which `hello` is answered
    /// `busy`.
    pub sessions: usize,
    /// How many connections may be open, sessions or not yet, past which
    /// they are closed at once.
    pub connections: usize,
    /// How many of a session's requests may wait for the daemon, past
    /// which its next messages are not read until replies go out.
    pub in_flight: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            hello: Duration::from_secs(5),
            write: Duration::from_secs(10),
            sessions: 32,
            connections: 64,
            in_flight: 32,
        }
    }
}
