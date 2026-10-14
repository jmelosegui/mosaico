//! What every mosaico process sets up on Windows before anything else.

use mosaico_core::system::ProcessSetup;

use crate::dpi;

/// [`ProcessSetup`] on Windows.
pub struct WindowsSetup;

impl ProcessSetup for WindowsSetup {
    /// Makes the process per-monitor DPI aware (V2), so it reads and
    /// applies window and monitor coordinates in physical pixels, the
    /// same in a CLI command as in the daemon, whatever the scaling of
    /// each display.
    fn init(&self) {
        dpi::enable_dpi_awareness();
    }
}
