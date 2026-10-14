use mosaico_core::pid;
use mosaico_core::{Instance, Result};

use crate::dpi;
use crate::instance_guard::InstanceGuard;

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

/// Runs the Mosaico daemon.
///
/// Starts background threads for the Win32 event loop (which also
/// handles global hotkeys), focus-follows-mouse and the IPC listener.
/// The main thread manages the tiling workspace.
///
/// The daemon has no console, so it reports through the log. An error
/// from the daemon loop is logged here as well as returned. Errors before
/// it, such as another daemon holding the lock, come before the logger
/// is set up and are only returned.
pub fn run() -> Result<()> {
    dpi::enable_dpi_awareness();
    clean_old_binary();
    let instance = Instance::current()?;
    let _guard = InstanceGuard::acquire(instance)?;
    pid::write_pid_file()?;
    mosaico_core::log_info!("Daemon started (PID: {})", std::process::id());

    let result = daemon_loop::daemon_loop(&crate::ipc::pipe_name(instance));
    if let Err(e) = &result {
        mosaico_core::log_error!("Daemon stopped: {e}");
    }

    let _ = pid::remove_pid_file();

    result
}

/// Removes a leftover `.exe.old` backup from a previous self-update.
///
/// The update command renames the current binary to `.exe.old` before
/// writing the new one. On Windows the `.old` file may still be locked
/// at update time, so we retry deletion here at the next daemon start.
fn clean_old_binary() {
    if let Ok(exe) = std::env::current_exe() {
        let old = exe.with_extension("exe.old");
        if old.exists() {
            let _ = std::fs::remove_file(&old);
        }
    }
}

// ipc_loop moved to daemon_ipc.rs
