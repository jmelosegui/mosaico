//! Named pipe transport for the CLI to daemon channel.
//!
//! Only the byte streams live here. Framing, versioning and the accept
//! loop are in [`mosaico_core::ipc`].

use std::fs::File;
use std::io::{self, Read, Write};
use std::os::windows::io::{AsRawHandle, FromRawHandle};

use mosaico_core::Instance;
use mosaico_core::ipc::{IpcClient, IpcConnection, IpcError, IpcServer};
use windows::Win32::Foundation::{
    ERROR_FILE_NOT_FOUND, ERROR_PIPE_BUSY, ERROR_PIPE_CONNECTED, HANDLE, INVALID_HANDLE_VALUE,
};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_SHARE_NONE, FlushFileBuffers, OPEN_EXISTING, PIPE_ACCESS_DUPLEX,
};
use windows::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe, PIPE_READMODE_BYTE, PIPE_TYPE_BYTE,
    PIPE_UNLIMITED_INSTANCES, PIPE_WAIT, WaitNamedPipeW,
};
use windows::core::HSTRING;

/// The pipe an instance's daemon listens on: `\\.\pipe\mosaico` for the
/// default instance, `\\.\pipe\mosaico-<name>` for a named one.
pub fn pipe_name(instance: &Instance) -> String {
    format!(r"\\.\pipe\{}", instance.endpoint_name())
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
        // SAFETY: CreateFileW opens an existing named pipe as a client.
        unsafe {
            CreateFileW(
                &self.name,
                GENERIC_READ_WRITE,
                FILE_SHARE_NONE,
                None,
                OPEN_EXISTING,
                Default::default(),
                None,
            )
        }
    }
}

impl IpcClient for PipeClient {
    type Stream = File;

    /// Checks for a listening pipe instance without connecting to it.
    fn is_daemon_running(&self) -> bool {
        // SAFETY: WaitNamedPipeW only checks whether a pipe instance is
        // available; 1 ms makes it return almost immediately.
        unsafe { WaitNamedPipeW(&self.name, 1).as_bool() }
    }

    fn connect(&self) -> Result<File, IpcError> {
        let handle = match self.open() {
            Ok(handle) => handle,
            Err(e) if e.code() == ERROR_FILE_NOT_FOUND.to_hresult() => {
                return Err(IpcError::NotRunning);
            }
            // The only instance is serving another client. Wait for it
            // to free up, then try once more.
            Err(e) if e.code() == ERROR_PIPE_BUSY.to_hresult() => {
                // SAFETY: WaitNamedPipeW blocks for at most BUSY_WAIT_MS.
                if !unsafe { WaitNamedPipeW(&self.name, BUSY_WAIT_MS) }.as_bool() {
                    return Err(IpcError::Io(io::Error::from(e)));
                }
                self.open().map_err(|e| IpcError::Io(io::Error::from(e)))?
            }
            Err(e) => return Err(IpcError::Io(io::Error::from(e))),
        };
        // SAFETY: the handle was just opened and nothing else owns it, so
        // the File takes sole ownership and closes it when dropped.
        Ok(unsafe { File::from_raw_handle(handle.0) })
    }
}

/// The daemon end of the pipe.
///
/// Each accepted connection gets a fresh pipe instance, which is what
/// lets the next client connect while one is being answered.
pub struct PipeServer {
    name: HSTRING,
}

impl PipeServer {
    /// A server listening on `name`, usually from [`pipe_name`].
    pub fn new(name: &str) -> Self {
        Self {
            name: HSTRING::from(name),
        }
    }
}

impl IpcServer for PipeServer {
    type Connection = PipeConnection;

    fn accept(&mut self) -> Result<PipeConnection, IpcError> {
        // SAFETY: CreateNamedPipeW creates a new instance of the pipe; the
        // result is checked against INVALID_HANDLE_VALUE below. The
        // default security descriptor only lets the creating user (and
        // administrators) open the pipe for writing.
        let handle = unsafe {
            CreateNamedPipeW(
                &self.name,
                PIPE_ACCESS_DUPLEX,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
                PIPE_UNLIMITED_INSTANCES,
                4096,
                4096,
                0,
                None,
            )
        };
        if handle == INVALID_HANDLE_VALUE {
            return Err(IpcError::Io(io::Error::last_os_error()));
        }
        // SAFETY: the handle is valid and owned by nobody else; the File
        // closes it when the connection is dropped.
        let file = unsafe { File::from_raw_handle(handle.0) };

        // SAFETY: ConnectNamedPipe blocks until a client connects. A
        // client that connected between creation and this call is
        // reported as ERROR_PIPE_CONNECTED, which is a success.
        match unsafe { ConnectNamedPipe(handle, None) } {
            Ok(()) => {}
            Err(e) if e.code() == ERROR_PIPE_CONNECTED.to_hresult() => {}
            Err(e) => return Err(IpcError::Io(io::Error::from(e))),
        }
        Ok(PipeConnection { file })
    }
}

/// One client connection on the daemon side.
pub struct PipeConnection {
    file: File,
}

impl Read for PipeConnection {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.file.read(buf)
    }
}

impl Write for PipeConnection {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

impl IpcConnection for PipeConnection {
    fn finish(mut self) -> Result<(), IpcError> {
        self.file.flush()?;
        let handle = HANDLE(self.file.as_raw_handle());
        // SAFETY: FlushFileBuffers blocks until the client has read all
        // data from the pipe. Without this, DisconnectNamedPipe would
        // discard the unread reply and the client would get error 233.
        unsafe {
            let _ = FlushFileBuffers(handle);
        }
        // SAFETY: DisconnectNamedPipe ends this instance's connection; the
        // handle itself is closed when the File is dropped.
        unsafe { DisconnectNamedPipe(handle) }.map_err(|e| IpcError::Io(io::Error::from(e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_instance_keeps_the_usual_pipe() {
        // Arrange
        let instance = Instance::new(None, "cfg", "data").unwrap();

        // Act / Assert
        assert_eq!(pipe_name(&instance), r"\\.\pipe\mosaico");
    }

    #[test]
    fn a_named_instance_gets_its_own_pipe() {
        // Arrange
        let instance = Instance::new(Some("test-7"), "cfg", "data").unwrap();

        // Act / Assert
        assert_eq!(pipe_name(&instance), r"\\.\pipe\mosaico-test-7");
    }
}
