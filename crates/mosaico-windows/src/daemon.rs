use mosaico_core::config::ConfigStore;
use mosaico_core::{Instance, Result};

#[path = "daemon_inputs.rs"]
mod daemon_inputs;
#[path = "daemon_ipc.rs"]
mod daemon_ipc;
#[path = "daemon_loop.rs"]
mod daemon_loop;
#[path = "daemon_loop_handlers.rs"]
mod daemon_loop_handlers;
#[path = "daemon_loop_pump.rs"]
mod daemon_loop_pump;
#[path = "daemon_loop_run.rs"]
mod daemon_loop_run;
#[path = "daemon_loop_task.rs"]
mod daemon_loop_task;
#[path = "daemon_threads.rs"]
mod daemon_threads;
#[path = "daemon_types.rs"]
mod daemon_types;

/// Runs the daemon loop of `instance` until the daemon is told to stop.
///
/// `instance` must be the instance this process resolved at startup, the
/// one [`Instance::current`] returns. The pipe comes from `instance`, as
/// do the lock and the PID file [`mosaico_core::lifecycle::run`] holds,
/// and the config files, which the loop reads, watches and updates
/// through the instance's [`ConfigStore`]. But the loop finds the logs
/// and the record of hidden windows through [`Instance::current`]. A loop
/// run on another instance would mix the files of both, so it fails with
/// `InstanceError::NotCurrent` before it starts.
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
    instance.require_current()?;
    let store = ConfigStore::for_instance(instance);
    daemon_loop::daemon_loop(&crate::ipc::pipe_name(instance), &store)
}

// ipc_loop moved to daemon_ipc.rs
