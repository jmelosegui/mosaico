//! The CLI's end of the named pipe.

use std::fs::File;
use std::io::{self, Read, Write};
use std::time::{Duration, Instant};

use mosaico_core::ipc::{IpcError, IpcStream};

use super::watchdog::with_deadline;

/// The CLI's end of one connection to the daemon.
pub struct PipeStream {
    pub(super) file: File,
    /// When reading the reply stops waiting for the daemon, if set.
    pub(super) read_deadline: Option<Instant>,
}

impl Read for PipeStream {
    /// Reads the daemon's reply, failing with `TimedOut` once the
    /// deadline set by `set_read_timeout` has passed.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let file = &mut self.file;
        with_deadline(self.read_deadline, || file.read(buf))
    }
}

impl Write for PipeStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

impl IpcStream for PipeStream {
    fn set_read_timeout(&mut self, timeout: Duration) -> Result<(), IpcError> {
        self.read_deadline = Some(Instant::now() + timeout);
        Ok(())
    }

    fn clear_read_timeout(&mut self) -> Result<(), IpcError> {
        self.read_deadline = None;
        Ok(())
    }
}

#[cfg(test)]
#[path = "ipc_subscription_tests.rs"]
mod tests;
