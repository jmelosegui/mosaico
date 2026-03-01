use mosaico_core::WindowResult;
use mosaico_core::pid;

use crate::dpi;

#[path = "daemon_ipc.rs"]
mod daemon_ipc;
#[path = "daemon_loop.rs"]
mod daemon_loop;
#[path = "daemon_loop_handlers.rs"]
mod daemon_loop_handlers;
#[path = "daemon_threads.rs"]
mod daemon_threads;
#[path = "daemon_types.rs"]
mod daemon_types;

/// Runs the Mosaico daemon.
///
/// Starts background threads for the Win32 event loop (which also
/// handles global hotkeys) and the IPC listener. The main thread
/// manages the tiling workspace.
pub fn run() -> WindowResult<()> {
    dpi::enable_dpi_awareness();
    clean_old_binary();
    pid::write_pid_file()?;
    eprintln!("Mosaico daemon started.");

    let result = daemon_loop::daemon_loop();

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
