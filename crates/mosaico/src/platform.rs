//! Picks the platform implementation of each core contract.
//!
//! Commands use the core traits and get their implementation from here,
//! so supporting another OS means adding a branch to this file rather
//! than touching every command.

use mosaico_core::Instance;
use mosaico_core::ipc::IpcClient;

/// The daemon instance this CLI talks to.
///
/// `main` checks the instance before running any command, so this only
/// fails if that check was skipped.
pub fn instance() -> &'static Instance {
    Instance::current().expect("the instance is validated at startup")
}

/// The channel to the daemon of this instance.
#[cfg(target_os = "windows")]
pub fn ipc() -> impl IpcClient {
    mosaico_windows::ipc::PipeClient::new(&mosaico_windows::ipc::pipe_name(instance()))
}
