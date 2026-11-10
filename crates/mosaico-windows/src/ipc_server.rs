//! The daemon end of the pipe: its instances and its connections.

use std::fs::File;
use std::io::{self, Read, Write};
use std::os::windows::io::{AsRawHandle, FromRawHandle};
use std::time::{Duration, Instant};

use mosaico_core::ipc::{IpcConnection, IpcError, IpcServer};
use windows::Win32::Foundation::{ERROR_PIPE_CONNECTED, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::Storage::FileSystem::{
    FILE_FLAG_FIRST_PIPE_INSTANCE, FILE_FLAGS_AND_ATTRIBUTES, FlushFileBuffers, PIPE_ACCESS_DUPLEX,
};
use windows::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe, PIPE_READMODE_BYTE,
    PIPE_REJECT_REMOTE_CLIENTS, PIPE_TYPE_BYTE, PIPE_UNLIMITED_INSTANCES, PIPE_WAIT,
};
use windows::core::HSTRING;

use super::security::PipeSecurity;
use super::watchdog::with_deadline;

/// The daemon end of the pipe.
///
/// A spare pipe instance is created as soon as a client connects, so
/// while one client is being answered the next one finds a free instance
/// and connects straight away instead of finding the pipe busy.
///
/// Every instance only lets in the user who runs the daemon, on this
/// machine, at no lower integrity than the daemon's (see the `security`
/// module). The first must be the first instance of its name, so a pipe
/// someone else created before the daemon is never joined.
pub struct PipeServer {
    name: HSTRING,
    /// The instance the next client will connect to.
    spare: Option<File>,
    /// Whether an instance has been created yet.
    created: bool,
}

impl PipeServer {
    /// A server listening on `name`, usually from [`pipe_name`].
    pub fn new(name: &str) -> Self {
        Self {
            name: HSTRING::from(name),
            spare: None,
            created: false,
        }
    }

    /// Creates one more instance of the pipe.
    pub(super) fn create_instance(&mut self) -> Result<File, IpcError> {
        let security = PipeSecurity::for_this_process()?;
        let first = if self.created {
            FILE_FLAGS_AND_ATTRIBUTES(0)
        } else {
            FILE_FLAG_FIRST_PIPE_INSTANCE
        };
        let attributes = security.attributes();
        // SAFETY: CreateNamedPipeW creates a new instance of the pipe; the
        // result is checked against INVALID_HANDLE_VALUE below. The
        // attributes point into `security`, which outlives the call.
        let handle = unsafe {
            CreateNamedPipeW(
                &self.name,
                PIPE_ACCESS_DUPLEX | first,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
                PIPE_UNLIMITED_INSTANCES,
                4096,
                4096,
                0,
                Some(&attributes),
            )
        };
        if handle == INVALID_HANDLE_VALUE {
            return Err(IpcError::Io(io::Error::last_os_error()));
        }
        self.created = true;
        // SAFETY: the handle is valid and owned by nobody else; the File
        // closes it when it is dropped.
        Ok(unsafe { File::from_raw_handle(handle.0) })
    }
}

impl IpcServer for PipeServer {
    type Connection = PipeConnection;

    fn accept(&mut self) -> Result<PipeConnection, IpcError> {
        let file = match self.spare.take() {
            Some(file) => file,
            None => self.create_instance()?,
        };
        let handle = HANDLE(file.as_raw_handle());

        // SAFETY: ConnectNamedPipe blocks until a client connects. A
        // client that connected between creation and this call is
        // reported as ERROR_PIPE_CONNECTED, which is a success.
        match unsafe { ConnectNamedPipe(handle, None) } {
            Ok(()) => {}
            Err(e) if e.code() == ERROR_PIPE_CONNECTED.to_hresult() => {}
            // Any other failure is about this one client, typically
            // ERROR_NO_DATA for a client that connected and closed its
            // end before we got to it. Only a failure to create an
            // instance means the pipe can no longer be served. Put a
            // fresh instance in place before this one is dropped, so the
            // pipe never disappears for the next client.
            Err(e) => {
                mosaico_core::log_debug!("IPC: client connection failed: {e}");
                self.spare = Some(self.create_instance()?);
                return Err(IpcError::Closed);
            }
        }

        // Leave a free instance behind for the next client before this
        // one is served. If it cannot be created now, the next accept
        // tries again and reports the error if it still fails.
        match self.create_instance() {
            Ok(spare) => self.spare = Some(spare),
            Err(e) => mosaico_core::log_warn!("IPC: could not prepare the next pipe instance: {e}"),
        }
        Ok(PipeConnection {
            file,
            read_deadline: None,
            write_deadline: None,
        })
    }
}

/// One client connection on the daemon side.
pub struct PipeConnection {
    file: File,
    /// When reads stop waiting for the client's request, if set.
    read_deadline: Option<Instant>,
    /// When writing the reply and waiting for it to be read stop, if set.
    write_deadline: Option<Instant>,
}

impl Read for PipeConnection {
    /// Reads from the client, failing with `TimedOut` once the deadline
    /// set by `set_read_timeout` has passed. The deadline covers the
    /// whole request, so a client sending a byte at a time cannot keep
    /// the connection alive past it.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let file = &mut self.file;
        with_deadline(self.read_deadline, || file.read(buf))
    }
}

impl Write for PipeConnection {
    /// Writes to the client. A reply larger than the pipe's buffer waits
    /// for the client to read, so it fails with `TimedOut` once the
    /// deadline set by `set_write_timeout` has passed.
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let file = &mut self.file;
        with_deadline(self.write_deadline, || file.write(buf))
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

impl IpcConnection for PipeConnection {
    fn set_read_timeout(&mut self, timeout: Duration) -> Result<(), IpcError> {
        self.read_deadline = Some(Instant::now() + timeout);
        Ok(())
    }

    fn set_write_timeout(&mut self, timeout: Duration) -> Result<(), IpcError> {
        self.write_deadline = Some(Instant::now() + timeout);
        Ok(())
    }

    fn finish(mut self) -> Result<(), IpcError> {
        self.file.flush()?;
        let handle = HANDLE(self.file.as_raw_handle());
        // FlushFileBuffers waits until the client has read the whole
        // reply. Without it, DisconnectNamedPipe would discard the unread
        // reply and the client would get error 233. A client that never
        // reads would keep it waiting, so it is bounded by the write
        // deadline; past it the connection is dropped, reply and all.
        let flushed = with_deadline(self.write_deadline, || {
            // SAFETY: FlushFileBuffers only waits on this pipe handle,
            // which stays open until the File is dropped.
            unsafe { FlushFileBuffers(handle) }.map_err(io::Error::from)
        });
        // SAFETY: DisconnectNamedPipe ends this instance's connection; the
        // handle itself is closed when the File is dropped.
        let disconnected =
            unsafe { DisconnectNamedPipe(handle) }.map_err(|e| IpcError::Io(io::Error::from(e)));
        match flushed {
            Err(e) if e.kind() == io::ErrorKind::TimedOut => Err(IpcError::Io(e)),
            _ => disconnected,
        }
    }
}
