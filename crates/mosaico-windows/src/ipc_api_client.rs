//! The client end of the bar API pipe, for `mosaico query`, `subscribe`
//! and `run`.

use std::os::windows::io::{FromRawHandle, OwnedHandle};

use mosaico_core::api::ApiConnection;
use mosaico_core::api::client::{ApiClient, ClientConnection};
use mosaico_core::ipc::IpcError;
use windows::Win32::Storage::FileSystem::FILE_FLAG_OVERLAPPED;
use windows::core::HSTRING;

use super::api_io::ApiPipeConnection;
use super::{check_server, open_client};

/// Reaches the bar API of the daemon listening on one pipe.
pub struct ApiPipeClient {
    name: HSTRING,
}

impl ApiPipeClient {
    /// A client for the bar API pipe `name`, usually from
    /// [`api_pipe_name`](super::api_pipe_name).
    pub fn new(name: &str) -> Self {
        Self {
            name: HSTRING::from(name),
        }
    }
}

impl ApiClient for ApiPipeClient {
    /// Opens the pipe for overlapped I/O, as the daemon's end is: on a
    /// handle opened for synchronous I/O, Windows runs one call at a time,
    /// so a read waiting for the daemon would hold up every write.
    fn connect(&self) -> Result<ClientConnection, IpcError> {
        let handle = open_client(&self.name, FILE_FLAG_OVERLAPPED)?;
        // SAFETY: the handle was just opened and nothing else owns it, so
        // the OwnedHandle takes sole ownership and closes it when dropped.
        let handle = unsafe { OwnedHandle::from_raw_handle(handle.0) };
        check_server(&handle)?;
        let (reader, writer) = ApiPipeConnection::new(handle)?.split()?;
        Ok(ClientConnection {
            reader: Box::new(reader),
            writer: Box::new(writer),
        })
    }
}
