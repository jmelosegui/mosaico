//! Named pipe transports: the CLI to daemon channel and the bar API's.
//!
//! Only the byte streams live here. Framing, versioning and the accept
//! loop are in [`mosaico_core::ipc`] and [`mosaico_core::api`]. This file
//! holds the CLI client's end; the daemon's is in `ipc_server.rs`, and
//! the bar API's in `ipc_api.rs` and `ipc_api_io.rs`.

use std::fs::File;
use std::io::{self, Read, Write};
use std::os::windows::io::FromRawHandle;
use std::time::{Duration, Instant};

use mosaico_core::Instance;
use mosaico_core::ipc::{IpcClient, IpcError, IpcStream};
use windows::Win32::Foundation::{
    ERROR_FILE_NOT_FOUND, ERROR_PIPE_BUSY, GetLastError, HANDLE, WIN32_ERROR,
};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_NONE, OPEN_EXISTING,
    SECURITY_IDENTIFICATION, SECURITY_SQOS_PRESENT,
};
use windows::Win32::System::Pipes::WaitNamedPipeW;
use windows::core::HSTRING;

pub use api::ApiPipeListener;
pub use api_io::{ApiPipeConnection, ApiPipeReader, ApiPipeWriter};
pub use server::{PipeConnection, PipeServer};

use security::check_server;
use watchdog::with_deadline;

#[path = "ipc_api.rs"]
mod api;
#[path = "ipc_api_io.rs"]
mod api_io;
#[path = "ipc_security.rs"]
mod security;
#[path = "ipc_server.rs"]
mod server;
#[path = "ipc_watchdog.rs"]
mod watchdog;

/// The pipe an instance's daemon listens on: `\\.\pipe\mosaico` for the
/// default instance, `\\.\pipe\mosaico-<name>` for a named one.
pub fn pipe_name(instance: &Instance) -> String {
    format!(r"\\.\pipe\{}", instance.endpoint_name())
}

/// The bar API pipe of an instance's daemon: `\\.\pipe\mosaico.bar` for
/// the default instance, `\\.\pipe\mosaico-<name>.bar` for a named one.
pub fn api_pipe_name(instance: &Instance) -> String {
    format!(r"\\.\pipe\{}", instance.api_endpoint_name())
}

/// How long a client waits for a daemon that is busy with another
/// client, in milliseconds.
const BUSY_WAIT_MS: u32 = 2000;

const GENERIC_READ_WRITE: u32 = 0x80000000 | 0x40000000;

/// The CLI end of the pipe.
pub struct PipeClient {
    name: HSTRING,
}

impl PipeClient {
    /// A client for the daemon listening on `name`, usually from
    /// [`pipe_name`].
    pub fn new(name: &str) -> Self {
        Self {
            name: HSTRING::from(name),
        }
    }

    fn open(&self) -> windows::core::Result<HANDLE> {
        // At the identification level, the server can learn who the
        // client is but never act as it, so a pipe that turns out not to
        // be the daemon's gets nothing from the connection.
        let flags = FILE_FLAGS_AND_ATTRIBUTES(SECURITY_SQOS_PRESENT.0 | SECURITY_IDENTIFICATION.0);
        // SAFETY: CreateFileW opens an existing named pipe as a client.
        unsafe {
            CreateFileW(
                &self.name,
                GENERIC_READ_WRITE,
                FILE_SHARE_NONE,
                None,
                OPEN_EXISTING,
                flags,
                None,
            )
        }
    }
}

impl IpcClient for PipeClient {
    type Stream = PipeStream;

    /// Checks whether the pipe exists, without connecting to it.
    ///
    /// `WaitNamedPipeW` fails with `ERROR_FILE_NOT_FOUND` when no
    /// instance of the pipe exists, which is the only case in which no
    /// daemon is listening. When every instance is busy with another
    /// client it times out instead, and the daemon is running.
    fn is_daemon_running(&self) -> bool {
        // SAFETY: WaitNamedPipeW only checks whether a pipe instance is
        // available; 1 ms makes it return almost immediately.
        if unsafe { WaitNamedPipeW(&self.name, 1) }.as_bool() {
            return true;
        }
        // SAFETY: GetLastError reads the error of the call just above.
        let error = unsafe { GetLastError() };
        error != ERROR_FILE_NOT_FOUND
    }

    fn connect(&self) -> Result<PipeStream, IpcError> {
        let handle = match self.open() {
            Ok(handle) => handle,
            Err(e) if e.code() == ERROR_FILE_NOT_FOUND.to_hresult() => {
                return Err(IpcError::NotRunning);
            }
            // The only instance is serving another client. Wait for it
            // to free up, then try once more. A daemon that keeps every
            // instance busy that long is not answering anyone.
            Err(e) if e.code() == ERROR_PIPE_BUSY.to_hresult() => {
                // SAFETY: WaitNamedPipeW blocks for at most BUSY_WAIT_MS.
                if !unsafe { WaitNamedPipeW(&self.name, BUSY_WAIT_MS) }.as_bool() {
                    // SAFETY: GetLastError reads the error of the call just above.
                    let error = unsafe { GetLastError() };
                    return Err(busy_wait_failure(error));
                }
                self.open().map_err(|e| IpcError::Io(io::Error::from(e)))?
            }
            Err(e) => return Err(IpcError::Io(io::Error::from(e))),
        };
        // SAFETY: the handle was just opened and nothing else owns it, so
        // the File takes sole ownership and closes it when dropped.
        let file = unsafe { File::from_raw_handle(handle.0) };
        check_server(&file)?;
        Ok(PipeStream {
            file,
            read_deadline: None,
        })
    }
}

/// The CLI's end of one connection to the daemon.
pub struct PipeStream {
    file: File,
    /// When reading the reply stops waiting for the daemon, if set.
    read_deadline: Option<Instant>,
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
}

/// What a failed wait for a busy pipe means for the client.
///
/// The pipe going away during the wait means the daemon exited, not
/// that it stopped answering.
fn busy_wait_failure(error: WIN32_ERROR) -> IpcError {
    if error == ERROR_FILE_NOT_FOUND {
        IpcError::NotRunning
    } else {
        IpcError::TimedOut
    }
}

#[cfg(test)]
#[path = "ipc_busy_tests.rs"]
mod busy_tests;
#[cfg(test)]
#[path = "ipc_tests.rs"]
mod tests;
