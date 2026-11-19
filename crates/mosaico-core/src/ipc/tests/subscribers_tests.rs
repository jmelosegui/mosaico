//! Subscribers, written to over in-memory connections from their own
//! threads.

use std::io::{self, Read, Write};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::*;
use crate::ipc::IpcError;

/// How long a test waits for a writer thread before failing.
const WAIT: Duration = Duration::from_secs(5);

/// A connection that keeps what was written and says when it finished.
struct FakeConnection {
    written: Arc<Mutex<Vec<u8>>>,
    finished: Sender<()>,
    /// Fail every write, as a client that went away does.
    broken: bool,
}

impl Read for FakeConnection {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        Ok(0)
    }
}

impl Write for FakeConnection {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.broken {
            return Err(io::ErrorKind::BrokenPipe.into());
        }
        self.written.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl IpcConnection for FakeConnection {
    fn finish(self) -> Result<(), IpcError> {
        let _ = self.finished.send(());
        Ok(())
    }
}

struct Client {
    written: Arc<Mutex<Vec<u8>>>,
    finished: Receiver<()>,
}

impl Client {
    /// The events written once the connection finished.
    fn events(&self) -> Vec<DesktopEvent> {
        self.finished
            .recv_timeout(WAIT)
            .expect("the subscriber's connection was finished");
        let bytes = self.written.lock().unwrap().clone();
        String::from_utf8(bytes)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }
}

fn connection(broken: bool) -> (FakeConnection, Client) {
    let written = Arc::new(Mutex::new(Vec::new()));
    let (finished, done) = channel();
    let connection = FakeConnection {
        written: written.clone(),
        finished,
        broken,
    };
    (
        connection,
        Client {
            written,
            finished: done,
        },
    )
}

fn paused(paused: bool) -> DesktopEvent {
    DesktopEvent::Paused { paused }
}

#[test]
fn a_subscriber_gets_the_events_published_after_it_opened() {
    // Arrange
    let subscribers = Subscribers::default();
    subscribers.publish(&[paused(true)]);
    subscribers.open(Vec::new());
    let (conn, client) = connection(false);

    // Act -- published before and after the connection is attached.
    subscribers.publish(&[paused(false)]);
    subscribers.attach(conn);
    subscribers.publish(&[DesktopEvent::ConfigReloaded]);
    subscribers.close();

    // Assert
    assert_eq!(
        client.events(),
        vec![paused(false), DesktopEvent::ConfigReloaded]
    );
}

#[test]
fn a_subscriber_only_gets_the_kinds_it_asked_for() {
    // Arrange
    let subscribers = Subscribers::default();
    subscribers.open(vec![EventKind::ConfigReloaded]);
    let (conn, client) = connection(false);
    subscribers.attach(conn);

    // Act
    subscribers.publish(&[paused(true), DesktopEvent::ConfigReloaded]);
    subscribers.close();

    // Assert
    assert_eq!(client.events(), vec![DesktopEvent::ConfigReloaded]);
}

#[test]
fn a_subscriber_that_falls_behind_is_told_and_dropped() {
    // Arrange -- nothing reads its queue until it is attached.
    let subscribers = Subscribers::default();
    subscribers.open(vec![EventKind::Paused]);
    let events: Vec<DesktopEvent> = (0..=QUEUE_LENGTH).map(|i| paused(i % 2 == 0)).collect();

    // Act
    subscribers.publish(&events);
    let dropped = subscribers.is_empty();
    let (conn, client) = connection(false);
    subscribers.attach(conn);

    // Assert -- the queue it had, then `lagged`.
    assert!(dropped);
    let got = client.events();
    assert_eq!(got.len(), QUEUE_LENGTH + 1);
    assert_eq!(got.last(), Some(&DesktopEvent::Lagged));
}

#[test]
fn a_subscriber_whose_client_went_away_is_dropped() {
    // Arrange
    let subscribers = Subscribers::default();
    subscribers.open(Vec::new());
    let (conn, client) = connection(true);
    subscribers.attach(conn);
    subscribers.publish(&[paused(true)]);

    // Act -- the writer ends on the failed write.
    let _ = client.finished.recv_timeout(Duration::from_millis(200));
    subscribers.publish(&[paused(false)]);

    // Assert
    assert!(subscribers.is_empty());
}

#[test]
fn a_connection_with_no_subscriber_waiting_is_closed() {
    // Arrange
    let subscribers = Subscribers::default();
    let (conn, client) = connection(false);

    // Act
    subscribers.attach(conn);

    // Assert
    assert_eq!(client.events(), vec![]);
    assert!(subscribers.is_empty());
}
