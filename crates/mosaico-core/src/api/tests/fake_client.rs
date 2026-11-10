//! A client's side of the in-memory connections, and fakes for what the
//! client commands read, print and watch.

use std::io::{self, BufRead, BufReader, Read, Write};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::api::client::{ApiClient, ClientConnection, OpenInput};
use crate::ipc::IpcError;
use crate::system::ParentWatch;

use super::fake_pipe::{Chan, FakeListener};

struct DialReader(Arc<Chan>);

impl Read for DialReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf, None)
    }
}

/// Dropped, it ends what the client sends, as closing a pipe does.
struct DialWriter(Arc<Chan>);

impl Write for DialWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf, None)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for DialWriter {
    fn drop(&mut self) {
        self.0.close();
    }
}

/// Reaches whichever daemon listens now, or none.
#[derive(Clone, Default)]
pub(crate) struct FakeApiClient {
    listener: Arc<Mutex<Option<FakeListener>>>,
}

impl FakeApiClient {
    pub fn to(listener: &FakeListener) -> Self {
        let client = Self::default();
        client.listen(Some(listener));
        client
    }

    /// From now on, reaches `listener`, or finds no daemon running.
    pub fn listen(&self, listener: Option<&FakeListener>) {
        *self.listener.lock().unwrap() = listener.cloned();
    }
}

impl ApiClient for FakeApiClient {
    fn connect(&self) -> Result<ClientConnection, IpcError> {
        let listener = self.listener.lock().unwrap().clone();
        let listener = listener.ok_or(IpcError::NotRunning)?;
        let (to_daemon, from_daemon) = listener.connect().into_ends();
        Ok(ClientConnection {
            reader: Box::new(DialReader(from_daemon)),
            writer: Box::new(DialWriter(to_daemon)),
        })
    }
}

/// A daemon that sends `script` and ends, whatever it is sent.
pub(crate) struct Scripted(pub &'static str);

impl ApiClient for Scripted {
    fn connect(&self) -> Result<ClientConnection, IpcError> {
        Ok(ClientConnection {
            reader: Box::new(io::Cursor::new(self.0.as_bytes())),
            writer: Box::new(io::sink()),
        })
    }
}

/// What a command printed, readable while it runs.
#[derive(Clone, Default)]
pub(crate) struct Printed(Arc<Mutex<Vec<u8>>>);

impl Printed {
    pub fn lines(&self) -> Vec<String> {
        let bytes = self.0.lock().unwrap().clone();
        String::from_utf8(bytes)
            .unwrap()
            .lines()
            .map(str::to_string)
            .collect()
    }

    /// The lines, once there are at least `count`, waiting up to two
    /// seconds.
    pub fn at_least(&self, count: usize) -> Vec<String> {
        let deadline = Instant::now() + Duration::from_secs(2);
        while self.lines().len() < count && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(5));
        }
        self.lines()
    }
}

impl Write for Printed {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// An input fed line by line, which ends when its sender is dropped.
struct LineInput {
    lines: Receiver<String>,
    pending: Vec<u8>,
}

impl Read for LineInput {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.pending.is_empty() {
            match self.lines.recv() {
                Ok(line) => self.pending = format!("{line}\n").into_bytes(),
                Err(_) => return Ok(0),
            }
        }
        let n = buf.len().min(self.pending.len());
        buf[..n].copy_from_slice(&self.pending[..n]);
        self.pending.drain(..n);
        Ok(n)
    }
}

/// An input and the sender that feeds it.
pub(crate) fn input() -> (Sender<String>, OpenInput) {
    let (sender, lines) = mpsc::channel();
    let input = LineInput {
        lines,
        pending: Vec::new(),
    };
    let open = move || Box::new(BufReader::new(input)) as Box<dyn BufRead>;
    (sender, Box::new(open))
}

/// A parent process that exits when told.
#[derive(Default)]
pub(crate) struct FakeParent {
    exited: Mutex<bool>,
    changed: Condvar,
}

impl FakeParent {
    pub fn exit(&self) {
        *self.exited.lock().unwrap() = true;
        self.changed.notify_all();
    }
}

impl ParentWatch for FakeParent {
    fn pid(&self) -> Option<u32> {
        Some(4242)
    }

    fn wait(&self) {
        let mut exited = self.exited.lock().unwrap();
        while !*exited {
            exited = self.changed.wait(exited).unwrap();
        }
    }
}
