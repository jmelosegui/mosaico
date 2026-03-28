//! Ctrl+C signal handler using `SetConsoleCtrlHandler`.
//!
//! Replaces the `ctrlc` crate with a direct Win32 API call.

use std::sync::mpsc::Sender;

use windows::Win32::System::Console::{CTRL_C_EVENT, SetConsoleCtrlHandler};

/// Registers a Ctrl+C handler that sends `()` on the given channel.
///
/// The handler fires once; subsequent Ctrl+C signals are ignored.
///
/// # Errors
///
/// Returns an error if the Win32 console handler could not be installed
/// or if `set_handler` has already been called.
pub fn set_handler(tx: Sender<()>) -> Result<(), String> {
    // SAFETY: SetConsoleCtrlHandler installs a process-wide console
    // control handler. The callback pointer is valid for the process
    // lifetime (static function).
    unsafe { SetConsoleCtrlHandler(Some(handler), true) }
        .map_err(|e| format!("failed to set Ctrl+C handler: {e}"))?;

    // Store the sender in a global so the callback can reach it.
    SENDER
        .set(tx)
        .map_err(|_| "Ctrl+C handler already registered".to_string())?;
    Ok(())
}

/// Global sender — written once by `set_handler`, read by the callback.
static SENDER: std::sync::OnceLock<Sender<()>> = std::sync::OnceLock::new();

unsafe extern "system" fn handler(ctrl_type: u32) -> windows::core::BOOL {
    if ctrl_type == CTRL_C_EVENT
        && let Some(tx) = SENDER.get()
    {
        let _ = tx.send(());
    }
    windows::core::BOOL(1)
}
