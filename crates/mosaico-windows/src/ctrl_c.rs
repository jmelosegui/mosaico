//! Ctrl+C signal handler using `SetConsoleCtrlHandler`.
//!
//! Replaces the `ctrlc` crate with a direct Win32 API call.

use std::sync::mpsc::Sender;

use mosaico_core::{Error, Result};
use windows::Win32::System::Console::{CTRL_C_EVENT, SetConsoleCtrlHandler};

use crate::error::OrPlatform;

/// Registers a Ctrl+C handler that sends `()` on the given channel.
///
/// The handler fires once; subsequent Ctrl+C signals are ignored.
///
/// # Errors
///
/// Returns an error if the Win32 console handler could not be installed
/// or if `set_handler` has already been called.
pub fn set_handler(tx: Sender<()>) -> Result<()> {
    // SAFETY: SetConsoleCtrlHandler installs a process-wide console
    // control handler. The callback pointer is valid for the process
    // lifetime (static function).
    unsafe { SetConsoleCtrlHandler(Some(handler), true) }.or_platform("set the Ctrl+C handler")?;

    // Store the sender in a global so the callback can reach it.
    SENDER
        .set(tx)
        .map_err(|_| Error::platform("set the Ctrl+C handler", "one is already set"))?;
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
