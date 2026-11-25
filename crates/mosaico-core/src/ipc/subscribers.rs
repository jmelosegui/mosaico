//! The daemon's side of event subscriptions: who listens, and getting
//! each event to them without ever waiting on a slow one.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, TrySendError, sync_channel};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::Duration;

use crate::desktop_events::{DesktopEvent, EventKind};

use super::IpcConnection;

/// How many events wait for a subscriber before it counts as fallen
/// behind and is dropped.
pub const QUEUE_LENGTH: usize = 256;

/// How long writing one event may wait for the subscriber to read.
pub const SUBSCRIBER_WRITE_TIMEOUT: Duration = Duration::from_secs(5);

/// One subscriber, as the daemon's thread sees it.
struct Listener {
    /// The kinds it asked for; empty means every kind.
    kinds: Vec<EventKind>,
    queue: SyncSender<Arc<str>>,
    /// Set when the queue was full, so its writer ends with `lagged`.
    lagged: Arc<AtomicBool>,
}

impl Listener {
    fn wants(&self, kind: EventKind) -> bool {
        self.kinds.is_empty() || self.kinds.contains(&kind)
    }
}

/// What a writer needs to serve one subscriber.
struct Pending {
    queue: Receiver<Arc<str>>,
    lagged: Arc<AtomicBool>,
}

#[derive(Default)]
struct Inner {
    listeners: Vec<Listener>,
    /// The subscriber opened last, until its connection is attached.
    pending: Option<Pending>,
}

/// Every client subscribed to desktop events. Cloning it shares it.
///
/// The daemon's thread opens a subscriber when it answers a
/// subscription, so no event that follows the snapshot in the reply is
/// missed, and publishes to every subscriber after each step. The IPC
/// thread then attaches the connection, and a thread of its own writes
/// the events. A subscriber whose queue fills is sent
/// [`DesktopEvent::Lagged`] and dropped, so a stuck client never holds up
/// the daemon.
#[derive(Clone, Default)]
pub struct Subscribers {
    inner: Arc<Mutex<Inner>>,
}

impl Subscribers {
    fn lock(&self) -> MutexGuard<'_, Inner> {
        // A writer thread never holds the lock, so it cannot be poisoned
        // by one; recover the data either way rather than lose events.
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Adds a subscriber for `kinds` (every kind when empty). Events
    /// published from now on wait in its queue until its connection is
    /// [attached](Self::attach).
    pub fn open(&self, kinds: Vec<EventKind>) {
        let (queue, receiver) = sync_channel(QUEUE_LENGTH);
        let lagged = Arc::new(AtomicBool::new(false));
        let mut inner = self.lock();
        inner.listeners.push(Listener {
            kinds,
            queue,
            lagged: lagged.clone(),
        });
        inner.pending = Some(Pending {
            queue: receiver,
            lagged,
        });
    }

    /// Starts writing the events of the subscriber opened last to
    /// `connection`, on a thread of its own. Without one waiting, the
    /// connection is closed.
    pub fn attach<C: IpcConnection + Send + 'static>(&self, connection: C) {
        let Some(pending) = self.lock().pending.take() else {
            let _ = connection.finish();
            return;
        };
        let spawned = thread::Builder::new()
            .name("mosaico-subscriber".into())
            .spawn(move || write_events(connection, pending));
        if let Err(e) = spawned {
            crate::log_warn!("IPC: could not start writing events to a subscriber: {e}");
        }
    }

    /// Whether nobody is subscribed, so the daemon need not look for
    /// changes.
    pub fn is_empty(&self) -> bool {
        self.lock().listeners.is_empty()
    }

    /// Queues `events` for every subscriber that wants them. Drops the
    /// subscribers that went away or fell behind.
    pub fn publish(&self, events: &[DesktopEvent]) {
        let mut inner = self.lock();
        for event in events {
            let line: Arc<str> = match serde_json::to_string(event) {
                Ok(line) => line.into(),
                Err(e) => {
                    crate::log_warn!("IPC: could not encode {event:?}: {e}");
                    continue;
                }
            };
            inner.listeners.retain(|listener| {
                if !listener.wants(event.kind()) {
                    return true;
                }
                match listener.queue.try_send(line.clone()) {
                    Ok(()) => true,
                    Err(TrySendError::Full(_)) => {
                        crate::log_warn!("IPC: dropped a subscriber that fell behind");
                        listener.lagged.store(true, Ordering::SeqCst);
                        false
                    }
                    Err(TrySendError::Disconnected(_)) => false,
                }
            });
        }
    }

    /// Ends every subscription, as when the daemon stops. Each writer
    /// sends what is queued, then closes its connection.
    pub fn close(&self) {
        let mut inner = self.lock();
        inner.listeners.clear();
        inner.pending = None;
    }
}

/// Writes each queued event as one line until the queue closes or the
/// client stops reading, then `lagged` if the subscriber fell behind.
fn write_events<C: IpcConnection>(mut connection: C, pending: Pending) {
    for line in pending.queue.iter() {
        if write_line(&mut connection, &line).is_err() {
            return;
        }
    }
    if pending.lagged.load(Ordering::SeqCst) {
        let lagged = serde_json::to_string(&DesktopEvent::Lagged).unwrap_or_default();
        if write_line(&mut connection, &lagged).is_err() {
            return;
        }
    }
    let _ = connection.set_write_timeout(SUBSCRIBER_WRITE_TIMEOUT);
    let _ = connection.finish();
}

fn write_line<C: IpcConnection>(connection: &mut C, line: &str) -> std::io::Result<()> {
    // Each event gets its own deadline, so a subscriber that stops
    // reading ends its own thread and nothing else.
    let _ = connection.set_write_timeout(SUBSCRIBER_WRITE_TIMEOUT);
    connection.write_all(line.as_bytes())?;
    connection.write_all(b"\n")?;
    connection.flush()
}

#[cfg(test)]
#[path = "tests/subscribers_tests.rs"]
mod tests;
