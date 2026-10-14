//! Ctrl+C signal handler using `SetConsoleCtrlHandler`.
//!
//! Replaces the `ctrlc` crate with a direct Win32 API call.

use std::sync::mpsc::Sender;

use mosaico_core::system::Interrupts;
use mosaico_core::{Error, Result};
use windows::Win32::System::Console::{CTRL_C_EVENT, SetConsoleCtrlHandler};

use crate::error::OrPlatform;

/// [`Interrupts`] on Windows: a console control handler.
///
/// The handler claims every console control event, so Ctrl+Break does
/// not end the process either, but it only sends for Ctrl+C. Closing the
/// console still ends the process once the handler returns, as Windows
/// does for that event. A second call fails with "one is already set" and
/// Ctrl+C keeps going to the first channel.
pub struct ConsoleInterrupts;

impl Interrupts for ConsoleInterrupts {
    fn on_interrupt(&self, tx: Sender<()>) -> Result<()> {
        set_handler(tx)
    }
}

/// Registers a Ctrl+C handler that sends `()` on the given channel.
///
/// Every Ctrl+C sends; once the receiver is gone, the send fails and the
/// Ctrl+C is ignored.
///
/// # Errors
///
/// Returns an error if the Win32 console handler could not be installed
/// or if `set_handler` has already been called.
fn set_handler(tx: Sender<()>) -> Result<()> {
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
