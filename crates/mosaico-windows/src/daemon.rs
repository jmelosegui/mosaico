use mosaico_core::{Instance, Result};

#[path = "daemon_inputs.rs"]
mod daemon_inputs;
#[path = "daemon_ipc.rs"]
mod daemon_ipc;
#[path = "daemon_loop.rs"]
mod daemon_loop;
#[path = "daemon_loop_handlers.rs"]
mod daemon_loop_handlers;
#[path = "daemon_settle.rs"]
mod daemon_settle;
#[path = "daemon_threads.rs"]
mod daemon_threads;
#[path = "daemon_types.rs"]
mod daemon_types;

/// Runs the daemon loop of `instance` until the daemon is told to stop.
///
/// This is the Windows part of the daemon, which
/// [`mosaico_core::lifecycle::run`] runs while it holds the instance's
/// lock and PID file. It starts background threads for the Win32 event
/// loop, global hotkeys, focus-follows-mouse and the IPC listener, and
/// the calling thread manages the tiling workspace.
///
/// The process must have been set up first, as `main` does for every
/// command, so the daemon sees the desktop in physical pixels.
pub fn serve(instance: &Instance) -> Result<()> {
    daemon_loop::daemon_loop(&crate::ipc::pipe_name(instance))
}

// ipc_loop moved to daemon_ipc.rs
