use std::io::{BufRead, BufReader, Write};
use std::os::windows::io::FromRawHandle;

use mosaico_core::WindowResult;
use mosaico_core::ipc::{Command, PIPE_NAME, Response};
use windows::Win32::Foundation::{
    CloseHandle, DUPLICATE_SAME_ACCESS, DuplicateHandle, HANDLE, INVALID_HANDLE_VALUE,
};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_SHARE_NONE, FlushFileBuffers, OPEN_EXISTING, PIPE_ACCESS_DUPLEX,
};
use windows::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe, PIPE_READMODE_BYTE, PIPE_TYPE_BYTE,
    PIPE_UNLIMITED_INSTANCES, PIPE_WAIT, WaitNamedPipeW,
};
use windows::Win32::System::Threading::GetCurrentProcess;
use windows::core::HSTRING;

const GENERIC_READ_WRITE: u32 = 0x80000000 | 0x40000000;

/// A Named Pipe server that the daemon uses to accept CLI connections.
///
/// The server creates a pipe, waits for a client to connect, reads one
/// command, and returns a response. Each connection handles one request.
pub struct PipeServer {
    handle: HANDLE,
}

impl PipeServer {
    /// Creates a new Named Pipe server.
    ///
    /// This creates the pipe but does not yet wait for connections.
    pub fn create() -> WindowResult<Self> {
        let pipe_name = HSTRING::from(PIPE_NAME);

        // SAFETY: CreateNamedPipeW creates a new named pipe instance.
        // We pass valid parameters and check for INVALID_HANDLE_VALUE.
        let handle = unsafe {
            CreateNamedPipeW(
                &pipe_name,
                PIPE_ACCESS_DUPLEX,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
                PIPE_UNLIMITED_INSTANCES,
                512, // output buffer size
                512, // input buffer size
                0,   // default timeout
                None,
            )
        };

        if handle == INVALID_HANDLE_VALUE {
            return Err("Failed to create named pipe".into());
        }

        Ok(Self { handle })
    }

    /// Blocks until a client connects, reads a command, and returns it.
    pub fn accept_command(&self) -> WindowResult<Command> {
        // SAFETY: ConnectNamedPipe blocks until a client connects.
        unsafe {
            ConnectNamedPipe(self.handle, None)?;
        }

        let reader = duplicate_handle_as_file(self.handle)?;
        let mut buf_reader = BufReader::new(reader);
        let mut line = String::new();
        buf_reader.read_line(&mut line)?;

        let command: Command = serde_json::from_str(line.trim())?;
        Ok(command)
    }

    /// Sends a response back to the connected client and disconnects.
    pub fn send_response(&self, response: &Response) -> WindowResult<()> {
        let mut writer = duplicate_handle_as_file(self.handle)?;
        let json = serde_json::to_string(response)?;
        writeln!(writer, "{json}")?;
        writer.flush()?;

        // SAFETY: FlushFileBuffers blocks until the client has read all
        // data from the pipe. Without this, DisconnectNamedPipe would
        // discard unread data, causing the client to get error 233.
        unsafe {
            let _ = FlushFileBuffers(self.handle);
        }

        // SAFETY: DisconnectNamedPipe disconnects the server side so it
        // can accept new connections.
        unsafe {
            DisconnectNamedPipe(self.handle)?;
        }

        Ok(())
    }
}

impl Drop for PipeServer {
    fn drop(&mut self) {
        // SAFETY: CloseHandle releases the pipe handle when the server
        // is dropped.
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}

/// RAII guard that closes a HANDLE on drop.
struct HandleGuard(HANDLE);

impl Drop for HandleGuard {
    fn drop(&mut self) {
        // SAFETY: CloseHandle releases the handle. The guard owns
        // this handle exclusively.
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

/// Sends a command to the daemon over the named pipe and returns the response.
///
/// This is used by the CLI (client side). The pipe handle is closed
/// automatically when the guard goes out of scope, even on error paths.
pub fn send_command(command: &Command) -> WindowResult<Response> {
    let pipe_name = HSTRING::from(PIPE_NAME);

    // SAFETY: CreateFileW opens an existing named pipe as a client.
    let handle = unsafe {
        CreateFileW(
            &pipe_name,
            GENERIC_READ_WRITE,
            FILE_SHARE_NONE,
            None,
            OPEN_EXISTING,
            Default::default(),
            None,
        )?
    };

    let _guard = HandleGuard(handle);

    let json = serde_json::to_string(command)?;

    // Write the command
    let mut writer = duplicate_handle_as_file(handle)?;
    writeln!(writer, "{json}")?;
    writer.flush()?;

    // Read the response
    let reader = duplicate_handle_as_file(handle)?;
    let mut buf_reader = BufReader::new(reader);
    let mut response_line = String::new();
    buf_reader.read_line(&mut response_line)?;

    let response: Response = serde_json::from_str(response_line.trim())?;
    Ok(response)
}

/// Checks if the daemon's named pipe exists (i.e. the daemon is running).
///
/// Uses `WaitNamedPipeW` with a 1 ms timeout instead of `CreateFileW`.
/// This avoids consuming a pipe connection — it only checks whether the
/// pipe exists without actually connecting to it.
pub fn is_daemon_running() -> bool {
    let pipe_name = HSTRING::from(PIPE_NAME);

    // SAFETY: WaitNamedPipeW checks whether a pipe instance is available.
    // A timeout of 1 ms means we return almost immediately.
    // Returns BOOL — as_bool() converts to a native Rust bool.
    unsafe { WaitNamedPipeW(&pipe_name, 1).as_bool() }
}

/// Duplicates a HANDLE and wraps it as a `std::fs::File`.
///
/// We duplicate instead of converting directly so that the original handle
/// and the File can be closed independently — avoids double-close bugs.
fn duplicate_handle_as_file(handle: HANDLE) -> WindowResult<std::fs::File> {
    let mut dup = HANDLE::default();

    // SAFETY: DuplicateHandle creates a copy of the handle. The duplicate
    // will be owned by the returned File and closed when it's dropped.
    unsafe {
        DuplicateHandle(
            GetCurrentProcess(),
            handle,
            GetCurrentProcess(),
            &mut dup,
            0,
            false,
            DUPLICATE_SAME_ACCESS,
        )?;

        Ok(std::fs::File::from_raw_handle(dup.0))
    }
}
