//! The daemon's bar API endpoint: a pipe of its own beside the CLI's,
//! which ordinary programs of the user may open even when the daemon runs
//! with more rights.

use std::io;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::sync::Arc;

use windows::Win32::Foundation::{
    ERROR_IO_PENDING, ERROR_PIPE_CONNECTED, HANDLE, INVALID_HANDLE_VALUE, WAIT_OBJECT_0,
};
use windows::Win32::Storage::FileSystem::{
    FILE_FLAG_FIRST_PIPE_INSTANCE, FILE_FLAG_OVERLAPPED, PIPE_ACCESS_DUPLEX,
};
use windows::Win32::System::IO::{CancelIoEx, GetOverlappedResult, OVERLAPPED};
use windows::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, PIPE_READMODE_BYTE, PIPE_REJECT_REMOTE_CLIENTS,
    PIPE_TYPE_BYTE, PIPE_UNLIMITED_INSTANCES, PIPE_WAIT,
};
use windows::Win32::System::Threading::{INFINITE, WaitForMultipleObjects};
use windows::core::HSTRING;

use mosaico_core::api::{ApiListener, Stopper};
use mosaico_core::ipc::IpcError;

use super::api_io::{ApiPipeConnection, Event};
use super::security::PipeSecurity;

/// How much the daemon may write ahead of a bar's reading, in bytes: a
/// few full states, so a bar busy for a moment does not hold up its
/// session.
const OUT_BUFFER: u32 = 256 * 1024;

/// How much a bar may write ahead of the daemon's reading, in bytes.
const IN_BUFFER: u32 = 16 * 1024;

/// The daemon end of the bar API pipe.
///
/// Like the CLI's pipe, a spare instance waits for the next bar while the
/// ones connected are served, the first instance must be the first of its
/// name, and only the user, on this machine, may connect. Unlike it, the
/// instances are labelled medium whatever the daemon's own level, since a
/// bar is an ordinary program of the user.
pub struct ApiPipeListener {
    name: HSTRING,
    spare: Option<OwnedHandle>,
    created: bool,
    /// Set, for good, to stop accepting.
    stop: Arc<Event>,
    /// Set when a bar connects to the instance being waited on.
    connected: Event,
}

impl ApiPipeListener {
    /// A listener on `name`, usually from
    /// [`api_pipe_name`](super::api_pipe_name).
    pub fn new(name: &str) -> io::Result<Self> {
        Ok(Self {
            name: HSTRING::from(name),
            spare: None,
            created: false,
            stop: Arc::new(Event::new()?),
            connected: Event::new()?,
        })
    }

    fn create_instance(&mut self) -> Result<OwnedHandle, IpcError> {
        let security = PipeSecurity::at_medium()?;
        let mut open = PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED;
        if !self.created {
            open |= FILE_FLAG_FIRST_PIPE_INSTANCE;
        }
        let attributes = security.attributes();
        // SAFETY: CreateNamedPipeW creates a new instance of the pipe; the
        // result is checked against INVALID_HANDLE_VALUE below. The
        // attributes point into `security`, which outlives the call.
        let handle = unsafe {
            CreateNamedPipeW(
                &self.name,
                open,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
                PIPE_UNLIMITED_INSTANCES,
                OUT_BUFFER,
                IN_BUFFER,
                0,
                Some(&attributes),
            )
        };
        if handle == INVALID_HANDLE_VALUE {
            return Err(IpcError::Io(io::Error::last_os_error()));
        }
        self.created = true;
        // SAFETY: the handle is valid and owned by nobody else.
        Ok(unsafe { OwnedHandle::from_raw_handle(handle.0) })
    }

    /// Waits for a bar to connect to `instance`; `Ok(false)` once stopped.
    fn connect(&self, instance: &OwnedHandle) -> io::Result<bool> {
        let handle = HANDLE(instance.as_raw_handle());
        let mut overlapped = OVERLAPPED {
            hEvent: self.connected.handle(),
            ..OVERLAPPED::default()
        };
        // SAFETY: `overlapped` lives until the call is over: it is waited
        // for below, or cancelled and then waited for.
        match unsafe { ConnectNamedPipe(handle, Some(&mut overlapped)) } {
            Ok(()) => return Ok(true),
            Err(e) if e.code() == ERROR_PIPE_CONNECTED.to_hresult() => return Ok(true),
            Err(e) if e.code() == ERROR_IO_PENDING.to_hresult() => {}
            Err(e) => return Err(io::Error::from(e)),
        }
        let events = [self.connected.handle(), self.stop.handle()];
        // SAFETY: waits on two events this listener owns.
        let waited = unsafe { WaitForMultipleObjects(&events, false, INFINITE) };
        if waited != WAIT_OBJECT_0 {
            // SAFETY: cancels only this call, waited for just below.
            let _ = unsafe { CancelIoEx(handle, Some(&overlapped)) };
        }
        let mut moved = 0;
        // SAFETY: waits for the call on `overlapped` to be over.
        let result = unsafe { GetOverlappedResult(handle, &overlapped, &mut moved, true) };
        match result {
            Ok(()) => Ok(waited == WAIT_OBJECT_0 || !self.stop.is_set()),
            Err(_) if self.stop.is_set() => Ok(false),
            Err(e) => Err(io::Error::from(e)),
        }
    }
}

impl ApiListener for ApiPipeListener {
    type Connection = ApiPipeConnection;

    fn accept(&mut self) -> Result<Option<ApiPipeConnection>, IpcError> {
        if self.stop.is_set() {
            return Ok(None);
        }
        let instance = match self.spare.take() {
            Some(instance) => instance,
            None => self.create_instance()?,
        };
        match self.connect(&instance) {
            Ok(true) => {}
            Ok(false) => return Ok(None),
            // A bar that connected and left before it was accepted. A
            // fresh instance takes this one's place before it is closed,
            // so the pipe never goes away.
            Err(e) => {
                mosaico_core::log_debug!("bar API: a connection failed: {e}");
                self.spare = Some(self.create_instance()?);
                return Err(IpcError::Closed);
            }
        }
        match self.create_instance() {
            Ok(spare) => self.spare = Some(spare),
            Err(e) => mosaico_core::log_warn!("bar API: could not prepare the next instance: {e}"),
        }
        Ok(Some(ApiPipeConnection::new(instance)?))
    }

    fn stopper(&self) -> Stopper {
        let stop = Arc::clone(&self.stop);
        Box::new(move || stop.set())
    }
}

#[cfg(test)]
#[path = "ipc_api_tests.rs"]
mod tests;
