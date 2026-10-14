//! Picks the platform implementation of each core contract.
//!
//! Commands use the core traits and get their implementation from here,
//! so supporting another OS means adding a branch to this file rather
//! than touching every command.

use mosaico_core::ipc::IpcClient;

/// The channel to the daemon of this session.
#[cfg(target_os = "windows")]
pub fn ipc() -> impl IpcClient {
    mosaico_windows::ipc::PipeClient::new(mosaico_windows::ipc::DEFAULT_PIPE_NAME)
}
