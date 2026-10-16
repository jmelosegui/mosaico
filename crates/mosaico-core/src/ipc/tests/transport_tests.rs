//! The accept loop and the client exchange, over in-memory streams.

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::io::{self, Cursor};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::*;
use crate::ipc::{MAX_MESSAGE_BYTES, PROTOCOL_VERSION, ResponseStatus};

/// One scripted connection: what the client sends, and what it got.
struct FakeConnection {
    input: Cursor<Vec<u8>>,
    output: Rc<RefCell<Vec<u8>>>,
    finished: Rc<Cell<bool>>,
}

impl Read for FakeConnection {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.input.read(buf)
    }
}

impl Write for FakeConnection {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.output.borrow_mut().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl IpcConnection for FakeConnection {
    fn finish(self) -> Result<(), IpcError> {
        self.finished.set(true);
        Ok(())
    }
}

/// What a client saw on its connection.
struct Seen {
    output: Rc<RefCell<Vec<u8>>>,
    finished: Rc<Cell<bool>>,
}

impl Seen {
    fn reply(&self) -> Option<Response> {
        let bytes = self.output.borrow().clone();
        if bytes.is_empty() {
            return None;
        }
        read_message(&mut BufReader::new(Cursor::new(bytes))).ok()
    }
}

/// A server that hands out scripted connections, then fails.
///
/// `None` in `pending` is a client that connected and left before
/// `accept` could hand it out.
#[derive(Default)]
struct FakeServer {
    pending: VecDeque<Option<FakeConnection>>,
}

impl FakeServer {
    fn client_sends(&mut self, bytes: &[u8]) -> Seen {
        let output = Rc::new(RefCell::new(Vec::new()));
        let finished = Rc::new(Cell::new(false));
        self.pending.push_back(Some(FakeConnection {
            input: Cursor::new(bytes.to_vec()),
            output: Rc::clone(&output),
            finished: Rc::clone(&finished),
        }));
        Seen { output, finished }
    }

    /// A client that connects and closes its end before `accept`
    /// returns, so the connection fails on the server side.
    fn client_leaves_before_accept(&mut self) {
        self.pending.push_back(None);
    }

    fn client_requests(&mut self, command: Command) -> Seen {
        let mut wire = Vec::new();
        write_message(&mut wire, &Request::new(command)).unwrap();
        self.client_sends(&wire)
    }
}

impl IpcServer for FakeServer {
    type Connection = FakeConnection;

    fn accept(&mut self) -> Result<FakeConnection, IpcError> {
        match self.pending.pop_front() {
            Some(Some(connection)) => Ok(connection),
            Some(None) => Err(IpcError::Closed),
            None => Err(IpcError::Io(io::Error::other("listener gone"))),
        }
    }
}

#[test]
fn commands_reach_the_handler_and_get_its_reply() {
    // Arrange
    let mut server = FakeServer::default();
    let status = server.client_requests(Command::Status);
    let stop = server.client_requests(Command::Stop);
    let mut handled = Vec::new();

    // Act
    let served = serve(&mut server, |command| {
        handled.push(command);
        Some(Response::ok_with_message("done"))
    });

    // Assert
    assert!(matches!(served, Served::Stopped));
    assert_eq!(handled, vec![Command::Status, Command::Stop]);
    assert_eq!(status.reply(), Some(Response::ok_with_message("done")));
    assert!(status.finished.get() && stop.finished.get());
}

#[test]
fn a_malformed_request_is_answered_without_reaching_the_handler() {
    // Arrange
    let mut server = FakeServer::default();
    let garbage = server.client_sends(b"this is not json\n");
    server.client_requests(Command::Stop);
    let mut handled = Vec::new();

    // Act
    serve(&mut server, |command| {
        handled.push(command);
        Some(Response::ok())
    });

    // Assert -- answered, and the daemon carried on to the next client.
    let reply = garbage.reply().expect("the client gets an answer");
    assert_eq!(reply.status, ResponseStatus::Error);
    assert_eq!(reply.code, Some(ErrorCode::Malformed));
    assert_eq!(handled, vec![Command::Stop]);
}

#[test]
fn an_unknown_command_is_answered_with_a_code() {
    // Arrange -- a newer CLI on the same protocol version.
    let mut server = FakeServer::default();
    let line = format!("{{\"version\":{PROTOCOL_VERSION},\"command\":\"Teleport\"}}\n");
    let unknown = server.client_sends(line.as_bytes());

    // Act
    serve(&mut server, |_| Some(Response::ok()));

    // Assert
    assert_eq!(
        unknown.reply().and_then(|r| r.code),
        Some(ErrorCode::UnknownCommand)
    );
}

#[test]
fn a_request_from_another_version_is_answered_with_a_mismatch() {
    // Arrange
    let mut server = FakeServer::default();
    let old = server.client_sends(b"{\"command\":\"Status\"}\n");

    // Act
    serve(&mut server, |_| Some(Response::ok()));

    // Assert
    assert_eq!(
        old.reply().and_then(|r| r.code),
        Some(ErrorCode::VersionMismatch)
    );
}

#[test]
fn a_reply_too_large_to_send_is_answered_with_too_large() {
    // #74
    // Arrange -- the answer to the first client does not fit in one
    // message; the next client asks the daemon to stop.
    let mut server = FakeServer::default();
    let big = server.client_requests(Command::Inspect);
    server.client_requests(Command::Stop);

    // Act
    let served = serve(&mut server, |command| {
        Some(match command {
            Command::Inspect => Response::ok_with_message("a".repeat(MAX_MESSAGE_BYTES)),
            _ => Response::ok(),
        })
    });

    // Assert -- the client is told why, and the daemon carries on.
    let reply = big.reply().expect("the client gets an answer");
    assert_eq!(reply.status, ResponseStatus::Error);
    assert_eq!(reply.code, Some(ErrorCode::TooLarge));
    assert!(big.finished.get(), "the connection is finished");
    assert!(matches!(served, Served::Stopped), "{served:?}");
}

#[test]
fn a_client_that_hangs_up_gets_no_reply_and_does_not_stop_the_daemon() {
    // Arrange
    let mut server = FakeServer::default();
    let gone = server.client_sends(b"");
    server.client_requests(Command::Stop);

    // Act
    let served = serve(&mut server, |_| Some(Response::ok()));

    // Assert
    assert!(gone.reply().is_none());
    assert!(matches!(served, Served::Stopped));
}

#[test]
fn a_client_that_leaves_before_it_is_accepted_does_not_stop_the_daemon() {
    // #72
    // Arrange -- one client connects and goes away before the server
    // picks it up; the next one asks for its status.
    let mut server = FakeServer::default();
    server.client_leaves_before_accept();
    let next = server.client_requests(Command::Status);
    server.client_requests(Command::Stop);
    let mut handled = Vec::new();

    // Act
    let served = serve(&mut server, |command| {
        handled.push(command);
        Some(Response::ok())
    });

    // Assert -- the daemon kept listening and answered the next client.
    assert!(matches!(served, Served::Stopped), "{served:?}");
    assert_eq!(handled, vec![Command::Status, Command::Stop]);
    assert_eq!(next.reply(), Some(Response::ok()));
}

#[test]
fn losing_the_listener_is_reported() {
    // Arrange -- no connections at all: the first accept fails.
    let mut server = FakeServer::default();

    // Act
    let served = serve(&mut server, |_| Some(Response::ok()));

    // Assert
    assert!(matches!(served, Served::Failed(_)));
}

#[test]
fn a_handler_with_nobody_behind_it_ends_the_loop() {
    // Arrange
    let mut server = FakeServer::default();
    server.client_requests(Command::Status);

    // Act
    let served = serve(&mut server, |_| None);

    // Assert
    assert!(matches!(served, Served::Abandoned));
}

/// A client whose daemon replies with fixed bytes.
struct FakeClient {
    reply: Vec<u8>,
    sent: Rc<RefCell<Vec<u8>>>,
}

struct FakeStream {
    input: Cursor<Vec<u8>>,
    sent: Rc<RefCell<Vec<u8>>>,
}

impl Read for FakeStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.input.read(buf)
    }
}

impl Write for FakeStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.sent.borrow_mut().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl IpcStream for FakeStream {}

impl IpcClient for FakeClient {
    type Stream = FakeStream;

    fn is_daemon_running(&self) -> bool {
        true
    }

    fn connect(&self) -> Result<FakeStream, IpcError> {
        Ok(FakeStream {
            input: Cursor::new(self.reply.clone()),
            sent: Rc::clone(&self.sent),
        })
    }
}

#[test]
fn the_client_sends_a_versioned_request_and_reads_the_reply() {
    // Arrange
    let mut reply = Vec::new();
    write_message(&mut reply, &Response::ok_with_message("running")).unwrap();
    let client = FakeClient {
        reply,
        sent: Rc::default(),
    };

    // Act
    let response = client.send(&Command::Status).unwrap();

    // Assert
    assert_eq!(response, Response::ok_with_message("running"));
    let sent: Request = read_message(&mut BufReader::new(Cursor::new(
        client.sent.borrow().clone(),
    )))
    .unwrap();
    assert_eq!(sent, Request::new(Command::Status));
}

#[test]
fn the_client_reports_a_daemon_that_died_mid_request() {
    // Arrange -- the connection closes without a reply.
    let client = FakeClient {
        reply: Vec::new(),
        sent: Rc::default(),
    };

    // Act
    let result = client.send(&Command::Status);

    // Assert
    assert!(matches!(result, Err(IpcError::Closed)));
}

// -- a client that never sends --

/// A connection for a server running on its own thread.
enum ThreadedConnection {
    /// A client that connected and never sends anything. Its reads wait
    /// for the read timeout set on it, or for an hour if none is set.
    Silent { timeout: Option<Duration> },
    /// A client that sends `input` and collects the reply in `output`.
    Talking {
        input: Cursor<Vec<u8>>,
        output: Arc<Mutex<Vec<u8>>>,
    },
    /// A client that sends `input` and never reads the reply, so
    /// `finish` waits for the write timeout set on it, or for an hour if
    /// none is set.
    NeverReads {
        input: Cursor<Vec<u8>>,
        write_timeout: Option<Duration>,
    },
}

impl Read for ThreadedConnection {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Silent { timeout: Some(t) } => {
                std::thread::sleep(*t);
                Err(io::ErrorKind::TimedOut.into())
            }
            Self::Silent { timeout: None } => {
                std::thread::sleep(Duration::from_secs(3600));
                Ok(0)
            }
            Self::Talking { input, .. } | Self::NeverReads { input, .. } => input.read(buf),
        }
    }
}

impl Write for ThreadedConnection {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Self::Silent { .. } | Self::NeverReads { .. } => Ok(buf.len()),
            Self::Talking { output, .. } => output.lock().unwrap().write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl IpcConnection for ThreadedConnection {
    fn finish(self) -> Result<(), IpcError> {
        match self {
            Self::NeverReads {
                write_timeout: Some(t),
                ..
            } => {
                std::thread::sleep(t);
                Err(IpcError::Io(io::ErrorKind::TimedOut.into()))
            }
            Self::NeverReads {
                write_timeout: None,
                ..
            } => {
                std::thread::sleep(Duration::from_secs(3600));
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn set_read_timeout(&mut self, timeout: Duration) -> Result<(), IpcError> {
        if let Self::Silent { timeout: slot } = self {
            *slot = Some(timeout);
        }
        Ok(())
    }

    fn set_write_timeout(&mut self, timeout: Duration) -> Result<(), IpcError> {
        if let Self::NeverReads { write_timeout, .. } = self {
            *write_timeout = Some(timeout);
        }
        Ok(())
    }
}

/// A server, safe to move to another thread, that hands out scripted
/// connections and then fails.
struct ThreadedServer {
    pending: VecDeque<ThreadedConnection>,
}

impl IpcServer for ThreadedServer {
    type Connection = ThreadedConnection;

    fn accept(&mut self) -> Result<ThreadedConnection, IpcError> {
        self.pending
            .pop_front()
            .ok_or_else(|| IpcError::Io(io::Error::other("listener gone")))
    }
}

fn talking(command: Command) -> (ThreadedConnection, Arc<Mutex<Vec<u8>>>) {
    let mut wire = Vec::new();
    write_message(&mut wire, &Request::new(command)).unwrap();
    let output = Arc::new(Mutex::new(Vec::new()));
    let connection = ThreadedConnection::Talking {
        input: Cursor::new(wire),
        output: Arc::clone(&output),
    };
    (connection, output)
}

#[test]
fn a_client_that_never_sends_does_not_block_the_next_one() {
    // #83
    // Arrange -- the first client connects and sends nothing; the next
    // one asks for the daemon's status.
    let (status, status_reply) = talking(Command::Status);
    let (stop, _) = talking(Command::Stop);
    let mut server = ThreadedServer {
        pending: VecDeque::from([ThreadedConnection::Silent { timeout: None }, status, stop]),
    };
    let (done_tx, done_rx) = std::sync::mpsc::channel();

    // Act
    std::thread::spawn(move || {
        let served = serve_with_timeout(&mut server, Duration::from_millis(50), |command| {
            Some(Response::ok_with_message(format!("{command:?}")))
        });
        let _ = done_tx.send(served);
    });

    // Assert -- the silent client is given up on and the next one is
    // answered, well within the time a person would wait.
    let served = done_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("the next client was never answered: serve is still waiting on the silent client");
    assert!(matches!(served, Served::Stopped), "{served:?}");
    let reply: Response = read_message(&mut BufReader::new(Cursor::new(
        status_reply.lock().unwrap().clone(),
    )))
    .unwrap();
    assert_eq!(reply, Response::ok_with_message("Status"));
}

#[test]
fn a_client_that_never_reads_its_reply_does_not_block_the_next_one() {
    // #85
    // Arrange -- the first client sends its request and never reads the
    // reply; the next one asks for the daemon's status.
    let mut wire = Vec::new();
    write_message(&mut wire, &Request::new(Command::Inspect)).unwrap();
    let never_reads = ThreadedConnection::NeverReads {
        input: Cursor::new(wire),
        write_timeout: None,
    };
    let (status, status_reply) = talking(Command::Status);
    let (stop, _) = talking(Command::Stop);
    let mut server = ThreadedServer {
        pending: VecDeque::from([never_reads, status, stop]),
    };
    let (done_tx, done_rx) = std::sync::mpsc::channel();

    // Act
    std::thread::spawn(move || {
        let served = serve_with_timeout(&mut server, Duration::from_millis(50), |command| {
            Some(Response::ok_with_message(format!("{command:?}")))
        });
        let _ = done_tx.send(served);
    });

    // Assert
    let served = done_rx.recv_timeout(Duration::from_secs(2)).expect(
        "the next client was never answered: serve is still waiting for the first one to read",
    );
    assert!(matches!(served, Served::Stopped), "{served:?}");
    let reply: Response = read_message(&mut BufReader::new(Cursor::new(
        status_reply.lock().unwrap().clone(),
    )))
    .unwrap();
    assert_eq!(reply, Response::ok_with_message("Status"));
}

// -- a daemon that never replies --

/// A client whose daemon takes the request and never replies.
struct UnresponsiveDaemon;

/// The stream to an unresponsive daemon. Reads wait for the read
/// timeout set on it, or for an hour if none is set.
struct UnresponsiveStream {
    timeout: Option<Duration>,
}

impl Read for UnresponsiveStream {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        match self.timeout {
            Some(t) => {
                std::thread::sleep(t);
                Err(io::ErrorKind::TimedOut.into())
            }
            None => {
                std::thread::sleep(Duration::from_secs(3600));
                Ok(0)
            }
        }
    }
}

impl Write for UnresponsiveStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl IpcStream for UnresponsiveStream {
    fn set_read_timeout(&mut self, timeout: Duration) -> Result<(), IpcError> {
        self.timeout = Some(timeout);
        Ok(())
    }
}

impl IpcClient for UnresponsiveDaemon {
    type Stream = UnresponsiveStream;

    fn is_daemon_running(&self) -> bool {
        true
    }

    fn connect(&self) -> Result<UnresponsiveStream, IpcError> {
        Ok(UnresponsiveStream { timeout: None })
    }
}

#[test]
fn a_daemon_that_never_replies_is_reported_instead_of_waited_on() {
    // #86
    // Arrange
    let (done_tx, done_rx) = std::sync::mpsc::channel();

    // Act
    std::thread::spawn(move || {
        let reply =
            UnresponsiveDaemon.send_with_timeout(&Command::Status, Duration::from_millis(50));
        let _ = done_tx.send(reply);
    });

    // Assert
    let reply = done_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("send is still waiting for a daemon that never replies");
    assert!(matches!(reply, Err(IpcError::TimedOut)), "{reply:?}");
}
