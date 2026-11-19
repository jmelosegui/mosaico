//! Subscribing from the client, and the daemon handing a subscribed
//! connection on instead of closing it.

use std::cell::{Cell, RefCell};
use std::io::{self, Cursor, Write};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::*;
use crate::desktop_events::DesktopEvent;
use crate::ipc::{
    ErrorCode, IpcConnection, IpcServer, Served, serve_with_subscriptions, write_message,
};
use crate::snapshot::DesktopSnapshot;

fn empty_desktop() -> DesktopSnapshot {
    DesktopSnapshot {
        monitors: Vec::new(),
        disconnected: Vec::new(),
        displaced_windows: 0,
        daemon_elevated: false,
        focused_monitor: 0,
        focused_window: None,
    }
}

/// A daemon that answers with fixed bytes and keeps what it was sent.
struct FakeClient {
    reply: Vec<u8>,
    sent: Arc<Mutex<Vec<u8>>>,
    timeout_cleared: Arc<AtomicBool>,
}

struct FakeStream {
    input: Cursor<Vec<u8>>,
    sent: Arc<Mutex<Vec<u8>>>,
    timeout_cleared: Arc<AtomicBool>,
}

impl Read for FakeStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.input.read(buf)
    }
}

impl Write for FakeStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.sent.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl IpcStream for FakeStream {
    fn clear_read_timeout(&mut self) -> Result<(), IpcError> {
        self.timeout_cleared.store(true, Ordering::SeqCst);
        Ok(())
    }
}

impl IpcClient for FakeClient {
    type Stream = FakeStream;

    fn is_daemon_running(&self) -> bool {
        true
    }

    fn connect(&self) -> Result<FakeStream, IpcError> {
        Ok(FakeStream {
            input: Cursor::new(self.reply.clone()),
            sent: Arc::clone(&self.sent),
            timeout_cleared: Arc::clone(&self.timeout_cleared),
        })
    }
}

fn client(reply: &Response, events: &[DesktopEvent]) -> FakeClient {
    let mut bytes = Vec::new();
    write_message(&mut bytes, reply).unwrap();
    for event in events {
        write_message(&mut bytes, event).unwrap();
    }
    FakeClient {
        reply: bytes,
        sent: Arc::default(),
        timeout_cleared: Arc::default(),
    }
}

#[test]
fn an_accepted_subscription_reads_the_desktop_then_its_events() {
    // Arrange
    let reply = Response::ok().with_data(&empty_desktop());
    let client = client(&reply, &[DesktopEvent::ConfigReloaded]);

    // Act
    let Ok(Subscribed::Accepted(mut subscription)) =
        subscribe(&client, vec![EventKind::ConfigReloaded])
    else {
        panic!("the subscription was not accepted");
    };

    // Assert
    assert_eq!(subscription.snapshot, empty_desktop());
    assert!(client.timeout_cleared.load(Ordering::SeqCst));
    let event = subscription.events.next_event().unwrap();
    assert_eq!(event, DesktopEvent::ConfigReloaded);
    assert!(matches!(
        subscription.events.next_event(),
        Err(IpcError::Closed)
    ));
    let sent: Request = read_message(&mut BufReader::new(Cursor::new(
        client.sent.lock().unwrap().clone(),
    )))
    .unwrap();
    assert_eq!(
        sent.command,
        Command::Subscribe {
            events: vec![EventKind::ConfigReloaded]
        }
    );
}

#[test]
fn a_refused_subscription_returns_the_reply() {
    // Arrange
    let reply = Response::error(ErrorCode::UnknownCommand, "no");
    let client = client(&reply, &[]);

    // Act
    let subscribed = subscribe(&client, Vec::new()).unwrap();

    // Assert
    assert!(matches!(subscribed, Subscribed::Refused(r) if r == reply));
    assert!(!client.timeout_cleared.load(Ordering::SeqCst));
}

/// A server with one client that subscribes, then nothing.
struct OneSubscriber(Option<Connection>);

struct Connection {
    input: Cursor<Vec<u8>>,
    output: Rc<RefCell<Vec<u8>>>,
    finished: Rc<Cell<bool>>,
}

impl Read for Connection {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.input.read(buf)
    }
}

impl Write for Connection {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.output.borrow_mut().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl IpcConnection for Connection {
    fn finish(self) -> Result<(), IpcError> {
        self.finished.set(true);
        Ok(())
    }
}

impl IpcServer for OneSubscriber {
    type Connection = Connection;

    fn accept(&mut self) -> Result<Connection, IpcError> {
        self.0
            .take()
            .ok_or_else(|| IpcError::Io(io::Error::other("listener gone")))
    }
}

/// What the daemon wrote to a connection, and whether it closed it.
type Watched = (Rc<RefCell<Vec<u8>>>, Rc<Cell<bool>>);

fn subscribing_server() -> (OneSubscriber, Watched) {
    let mut input = Vec::new();
    let command = Command::Subscribe { events: Vec::new() };
    write_message(&mut input, &Request::new(command)).unwrap();
    let output = Rc::new(RefCell::new(Vec::new()));
    let finished = Rc::new(Cell::new(false));
    let connection = Connection {
        input: Cursor::new(input),
        output: Rc::clone(&output),
        finished: Rc::clone(&finished),
    };
    (OneSubscriber(Some(connection)), (output, finished))
}

#[test]
fn the_daemon_hands_an_accepted_subscription_on_open() {
    // Arrange
    let (mut server, (output, finished)) = subscribing_server();
    let mut handed = Vec::new();

    // Act
    let served = serve_with_subscriptions(
        &mut server,
        Duration::from_secs(1),
        |_| Some(Response::ok().with_data(&empty_desktop())),
        |connection| handed.push(connection),
    );

    // Assert
    assert!(matches!(served, Served::Failed(_)));
    assert_eq!(handed.len(), 1);
    assert!(!finished.get());
    let reply: Response = read_message(&mut BufReader::new(Cursor::new(output.take()))).unwrap();
    assert_eq!(reply.data::<DesktopSnapshot>(), Some(empty_desktop()));
}

#[test]
fn the_daemon_closes_a_refused_subscription() {
    // Arrange
    let (mut server, (_, finished)) = subscribing_server();
    let mut handed = 0;

    // Act
    serve_with_subscriptions(
        &mut server,
        Duration::from_secs(1),
        |_| Some(Response::error(ErrorCode::Failed, "no")),
        |_| handed += 1,
    );

    // Assert
    assert_eq!(handed, 0);
    assert!(finished.get());
}
