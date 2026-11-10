//! Overlapped reads and writes on one bar API pipe connection, each half
//! waiting on its own and giving up at a deadline or at shutdown.
//!
//! A handle opened for synchronous I/O runs one call at a time, so a
//! read waiting for the bar would hold up every write to it. The bar
//! API's instances are opened with `FILE_FLAG_OVERLAPPED` instead: each
//! call starts, then waits on its own event beside the connection's
//! shutdown event. A call given up on is cancelled with `CancelIoEx`, and
//! its result is waited for before its buffer and `OVERLAPPED` go away.

use std::io::{self, Read, Write};
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::sync::Arc;
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{
    ERROR_BROKEN_PIPE, ERROR_IO_PENDING, ERROR_NO_DATA, ERROR_PIPE_NOT_CONNECTED, HANDLE,
    WAIT_OBJECT_0, WAIT_TIMEOUT,
};
use windows::Win32::Storage::FileSystem::{ReadFile, WriteFile};
use windows::Win32::System::IO::{CancelIoEx, GetOverlappedResult, OVERLAPPED};
use windows::Win32::System::Threading::{
    CreateEventW, INFINITE, SetEvent, WaitForMultipleObjects, WaitForSingleObject,
};

use mosaico_core::api::{ApiConnection, ApiReader, ApiWriter};
use mosaico_core::ipc::IpcError;

/// A manual-reset event, closed when dropped.
pub(super) struct Event(OwnedHandle);

impl Event {
    /// A new event, not set.
    pub(super) fn new() -> io::Result<Self> {
        // SAFETY: CreateEventW makes a new unnamed event; the handle is
        // owned by the OwnedHandle, which closes it once.
        let handle = unsafe { CreateEventW(None, true, false, None) }?;
        // SAFETY: the handle was just created and nothing else owns it.
        Ok(Self(unsafe { OwnedHandle::from_raw_handle(handle.0) }))
    }

    pub(super) fn handle(&self) -> HANDLE {
        HANDLE(self.0.as_raw_handle())
    }

    /// Sets the event, waking whatever waits on it, for good.
    pub(super) fn set(&self) {
        // SAFETY: the handle is an event this value owns.
        let _ = unsafe { SetEvent(self.handle()) };
    }

    /// Whether the event is set.
    pub(super) fn is_set(&self) -> bool {
        // SAFETY: waiting no time on an event this value owns.
        let waited = unsafe { WaitForSingleObject(self.handle(), 0) };
        waited == WAIT_OBJECT_0
    }
}

/// What both halves of a connection share: the pipe instance, closed
/// once both are dropped, and the event that ends it.
struct Pipe {
    handle: OwnedHandle,
    shutdown: Event,
}

/// How an overlapped call ended when it did not complete.
enum GaveUp {
    TimedOut,
    Shutdown,
}

impl Pipe {
    fn handle(&self) -> HANDLE {
        HANDLE(self.handle.as_raw_handle())
    }

    /// Runs the call `start` begins on `overlapped`, waiting on `event`
    /// until it completes, the connection shuts down or `deadline` passes.
    /// Returns how many bytes it moved.
    fn run(
        &self,
        event: &Event,
        deadline: Option<Instant>,
        start: impl FnOnce(HANDLE, *mut OVERLAPPED) -> windows::core::Result<()>,
    ) -> io::Result<Result<u32, GaveUp>> {
        let mut overlapped = OVERLAPPED {
            hEvent: event.handle(),
            ..OVERLAPPED::default()
        };
        match start(self.handle(), &mut overlapped) {
            Ok(()) => {}
            Err(e) if e.code() == ERROR_IO_PENDING.to_hresult() => {
                let waited = self.wait(event, deadline);
                if let Some(gave_up) = waited {
                    // SAFETY: cancels only this call, whose OVERLAPPED
                    // lives until its result is read just below.
                    let _ = unsafe { CancelIoEx(self.handle(), Some(&overlapped)) };
                    return Ok(self.result(&overlapped, true).map_err(|_| gave_up));
                }
            }
            Err(e) => return Err(io::Error::from(e)),
        }
        self.result(&overlapped, true).map(Ok)
    }

    /// Waits on `event` and the shutdown; `None` once `event` is set.
    fn wait(&self, event: &Event, deadline: Option<Instant>) -> Option<GaveUp> {
        let handles = [event.handle(), self.shutdown.handle()];
        let left = deadline.map(|d| d.saturating_duration_since(Instant::now()));
        let ms = left.map_or(INFINITE, |left| millis(left).min(INFINITE - 1));
        // SAFETY: waits on two events owned by this connection.
        match unsafe { WaitForMultipleObjects(&handles, false, ms) } {
            WAIT_OBJECT_0 => None,
            WAIT_TIMEOUT => Some(GaveUp::TimedOut),
            _ => Some(GaveUp::Shutdown),
        }
    }

    /// The result of the call on `overlapped`, waiting for it if `wait`.
    fn result(&self, overlapped: &OVERLAPPED, wait: bool) -> io::Result<u32> {
        let mut moved = 0;
        // SAFETY: `overlapped` belongs to a call on this handle, and lives
        // until the call is over, which `wait` makes sure of.
        unsafe { GetOverlappedResult(self.handle(), overlapped, &mut moved, wait) }?;
        Ok(moved)
    }
}

/// Whole milliseconds in `duration`, rounded up so a wait never ends
/// before the deadline it was given.
fn millis(duration: Duration) -> u32 {
    let ms = duration.as_nanos().div_ceil(1_000_000);
    u32::try_from(ms).unwrap_or(u32::MAX)
}

/// Whether `error` means the bar closed its end.
fn gone(error: &io::Error) -> bool {
    let closed = [ERROR_BROKEN_PIPE, ERROR_NO_DATA, ERROR_PIPE_NOT_CONNECTED];
    (error.raw_os_error()).is_some_and(|code| closed.iter().any(|c| c.0 as i32 == code))
}

/// The daemon's end of one bar API connection.
pub struct ApiPipeConnection {
    pipe: Pipe,
}

impl ApiPipeConnection {
    /// A connection on the pipe instance `handle`, opened for overlapped
    /// I/O and connected.
    pub(super) fn new(handle: OwnedHandle) -> io::Result<Self> {
        let shutdown = Event::new()?;
        Ok(Self {
            pipe: Pipe { handle, shutdown },
        })
    }
}

impl ApiConnection for ApiPipeConnection {
    type Reader = ApiPipeReader;
    type Writer = ApiPipeWriter;

    fn split(self) -> Result<(ApiPipeReader, ApiPipeWriter), IpcError> {
        let pipe = Arc::new(self.pipe);
        let reader = ApiPipeReader {
            pipe: Arc::clone(&pipe),
            event: Event::new()?,
            deadline: None,
        };
        let writer = ApiPipeWriter {
            pipe,
            event: Event::new()?,
            deadline: None,
        };
        Ok((reader, writer))
    }
}

/// Reads what the bar sends.
pub struct ApiPipeReader {
    pipe: Arc<Pipe>,
    event: Event,
    deadline: Option<Instant>,
}

impl Read for ApiPipeReader {
    /// Reads what the bar sent; nothing, as at its end, once the
    /// connection shuts down or the bar closes its end.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.pipe.shutdown.is_set() || buf.is_empty() {
            return Ok(0);
        }
        let read = self
            .pipe
            .run(&self.event, self.deadline, |handle, overlapped| {
                // SAFETY: `buf` and `overlapped` outlive the call, which `run`
                // waits for before returning.
                unsafe { ReadFile(handle, Some(buf), None, Some(overlapped)) }
            });
        match read {
            Ok(Ok(n)) => Ok(n as usize),
            Ok(Err(GaveUp::TimedOut)) => Err(io::ErrorKind::TimedOut.into()),
            Ok(Err(GaveUp::Shutdown)) => Ok(0),
            Err(e) if gone(&e) => Ok(0),
            Err(e) => Err(e),
        }
    }
}

impl ApiReader for ApiPipeReader {
    fn set_read_timeout(&mut self, timeout: Option<Duration>) -> io::Result<()> {
        self.deadline = timeout.map(|timeout| Instant::now() + timeout);
        Ok(())
    }
}

/// Writes to the bar.
pub struct ApiPipeWriter {
    pipe: Arc<Pipe>,
    event: Event,
    deadline: Option<Instant>,
}

impl Write for ApiPipeWriter {
    /// Writes to the bar, waiting while the pipe is full until the bar
    /// reads, the deadline passes or the connection shuts down.
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.pipe.shutdown.is_set() {
            return Err(io::ErrorKind::BrokenPipe.into());
        }
        let written = self
            .pipe
            .run(&self.event, self.deadline, |handle, overlapped| {
                // SAFETY: `buf` and `overlapped` outlive the call, which `run`
                // waits for before returning.
                unsafe { WriteFile(handle, Some(buf), None, Some(overlapped)) }
            });
        match written? {
            Ok(n) => Ok(n as usize),
            Err(GaveUp::TimedOut) => Err(io::ErrorKind::TimedOut.into()),
            Err(GaveUp::Shutdown) => Err(io::ErrorKind::BrokenPipe.into()),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl ApiWriter for ApiPipeWriter {
    fn set_write_timeout(&mut self, timeout: Duration) -> io::Result<()> {
        self.deadline = Some(Instant::now() + timeout);
        Ok(())
    }

    /// Wakes the reader and refuses further reads and writes. The instance
    /// is closed, not disconnected, once both halves are dropped, so what
    /// was written stays readable to the bar.
    fn shutdown(&mut self) {
        self.pipe.shutdown.set();
    }
}
