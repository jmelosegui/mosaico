//! In-memory connections for the session tests: byte queues with a
//! capacity, deadlines and shutdown, as the contracts in `connection.rs`
//! ask of a platform.

use std::collections::VecDeque;
use std::io::{self, Read, Write};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::api::{ApiConnection, ApiListener, ApiReader, ApiWriter, Stopper};
use crate::ipc::IpcError;

/// One direction of a connection.
pub(crate) struct Chan {
    state: Mutex<ChanState>,
    changed: Condvar,
}

struct ChanState {
    bytes: VecDeque<u8>,
    closed: bool,
    capacity: usize,
}

impl Chan {
    fn new(capacity: usize) -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(ChanState {
                bytes: VecDeque::new(),
                closed: false,
                capacity,
            }),
            changed: Condvar::new(),
        })
    }

    pub(crate) fn read(&self, buf: &mut [u8], deadline: Option<Instant>) -> io::Result<usize> {
        let mut state = self.state.lock().unwrap();
        loop {
            if !state.bytes.is_empty() {
                let n = buf.len().min(state.bytes.len());
                for (slot, byte) in buf.iter_mut().zip(state.bytes.drain(..n)) {
                    *slot = byte;
                }
                self.changed.notify_all();
                return Ok(n);
            }
            if state.closed {
                return Ok(0);
            }
            state = wait(&self.changed, state, deadline)?;
        }
    }

    pub(crate) fn write(&self, buf: &[u8], deadline: Option<Instant>) -> io::Result<usize> {
        let mut state = self.state.lock().unwrap();
        loop {
            if state.closed {
                return Err(io::ErrorKind::BrokenPipe.into());
            }
            let room = state.capacity.saturating_sub(state.bytes.len());
            if room > 0 {
                let n = room.min(buf.len());
                state.bytes.extend(&buf[..n]);
                self.changed.notify_all();
                return Ok(n);
            }
            state = wait(&self.changed, state, deadline)?;
        }
    }

    pub(crate) fn close(&self) {
        self.state.lock().unwrap().closed = true;
        self.changed.notify_all();
    }
}

fn wait<'a>(
    changed: &Condvar,
    state: std::sync::MutexGuard<'a, ChanState>,
    deadline: Option<Instant>,
) -> io::Result<std::sync::MutexGuard<'a, ChanState>> {
    let Some(deadline) = deadline else {
        return Ok(changed.wait(state).unwrap());
    };
    let left = deadline.saturating_duration_since(Instant::now());
    if left.is_zero() {
        return Err(io::ErrorKind::TimedOut.into());
    }
    Ok(changed.wait_timeout(state, left).unwrap().0)
}

/// The daemon's end of an in-memory connection. Dropped before it is
/// split, it ends the connection, as closing a real one does.
pub struct FakeConnection {
    ends: Option<(Arc<Chan>, Arc<Chan>)>,
}

impl Drop for FakeConnection {
    fn drop(&mut self) {
        if let Some((inbound, outbound)) = self.ends.take() {
            inbound.close();
            outbound.close();
        }
    }
}

pub struct FakeReader {
    inbound: Arc<Chan>,
    deadline: Option<Instant>,
}

pub struct FakeWriter {
    inbound: Arc<Chan>,
    outbound: Arc<Chan>,
    deadline: Option<Instant>,
}

impl ApiConnection for FakeConnection {
    type Reader = FakeReader;
    type Writer = FakeWriter;

    fn split(mut self) -> Result<(FakeReader, FakeWriter), IpcError> {
        let (inbound, outbound) = self.ends.take().unwrap();
        let reader = FakeReader {
            inbound: Arc::clone(&inbound),
            deadline: None,
        };
        let writer = FakeWriter {
            inbound,
            outbound,
            deadline: None,
        };
        Ok((reader, writer))
    }
}

impl Read for FakeReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inbound.read(buf, self.deadline)
    }
}

impl ApiReader for FakeReader {
    fn set_read_timeout(&mut self, timeout: Option<Duration>) -> io::Result<()> {
        self.deadline = timeout.map(|t| Instant::now() + t);
        Ok(())
    }
}

impl Write for FakeWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.outbound.write(buf, self.deadline)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl ApiWriter for FakeWriter {
    fn set_write_timeout(&mut self, timeout: Duration) -> io::Result<()> {
        self.deadline = Some(Instant::now() + timeout);
        Ok(())
    }

    fn shutdown(&mut self) {
        self.inbound.close();
        self.outbound.close();
    }
}

/// A bar's end of an in-memory connection.
pub struct FakeBar {
    to_daemon: Arc<Chan>,
    from_daemon: Arc<Chan>,
    pending: Vec<u8>,
}

impl FakeBar {
    /// Sends one line.
    pub fn send(&self, line: &str) {
        let line = format!("{line}\n");
        let mut rest = line.as_bytes();
        while !rest.is_empty() {
            let n = self.to_daemon.write(rest, None).unwrap();
            rest = &rest[n..];
        }
    }

    /// The next message, or `None` once the daemon ended the connection
    /// or nothing came within two seconds.
    pub fn recv(&mut self) -> Option<Value> {
        self.recv_within(Duration::from_secs(2))
    }

    /// The next message, or `None` when nothing comes within `timeout`.
    pub fn recv_within(&mut self, timeout: Duration) -> Option<Value> {
        let deadline = Instant::now() + timeout;
        loop {
            if let Some(end) = self.pending.iter().position(|b| *b == b'\n') {
                let line: Vec<u8> = self.pending.drain(..=end).collect();
                return Some(serde_json::from_slice(&line).unwrap());
            }
            let mut buf = [0; 4096];
            match self.from_daemon.read(&mut buf, Some(deadline)) {
                Ok(0) | Err(_) => return None,
                Ok(n) => self.pending.extend_from_slice(&buf[..n]),
            }
        }
    }

    /// Whether the daemon ended the connection, once what it sent is read.
    pub fn ended(&mut self) -> bool {
        self.recv().is_none() && self.from_daemon.state.lock().unwrap().closed
    }

    /// Goes away.
    pub fn leave(&self) {
        self.to_daemon.close();
    }

    /// Its two directions, toward the daemon and from it, for a client
    /// that reads and writes them itself.
    pub fn into_ends(self) -> (Arc<Chan>, Arc<Chan>) {
        (self.to_daemon, self.from_daemon)
    }
}

type Queue = Arc<(Mutex<(VecDeque<FakeConnection>, bool)>, Condvar)>;

/// Hands out the connections [`FakeListener::connect`] makes.
#[derive(Clone)]
pub struct FakeListener {
    queue: Queue,
}

impl FakeListener {
    pub fn new() -> Self {
        Self {
            queue: Arc::new((Mutex::new((VecDeque::new(), false)), Condvar::new())),
        }
    }

    /// Connects a bar whose end holds `capacity` bytes the daemon sent.
    pub fn connect_with(&self, capacity: usize) -> FakeBar {
        let to_daemon = Chan::new(usize::MAX);
        let from_daemon = Chan::new(capacity);
        let (lock, changed) = &*self.queue;
        lock.lock().unwrap().0.push_back(FakeConnection {
            ends: Some((Arc::clone(&to_daemon), Arc::clone(&from_daemon))),
        });
        changed.notify_all();
        FakeBar {
            to_daemon,
            from_daemon,
            pending: Vec::new(),
        }
    }

    /// Connects a bar.
    pub fn connect(&self) -> FakeBar {
        self.connect_with(usize::MAX)
    }
}

impl ApiListener for FakeListener {
    type Connection = FakeConnection;

    fn accept(&mut self) -> Result<Option<FakeConnection>, IpcError> {
        let (lock, changed) = &*self.queue;
        let mut queue = lock.lock().unwrap();
        loop {
            if queue.1 {
                return Ok(None);
            }
            if let Some(connection) = queue.0.pop_front() {
                return Ok(Some(connection));
            }
            queue = changed.wait(queue).unwrap();
        }
    }

    fn stopper(&self) -> Stopper {
        let queue = Arc::clone(&self.queue);
        Box::new(move || {
            queue.0.lock().unwrap().1 = true;
            queue.1.notify_all();
        })
    }
}
