//! What the daemon reads from the tiling manager that is built on the
//! Windows side: the `debug state` snapshot.

use mosaico_core::snapshot::DesktopSnapshot;

use super::{TilingManager, inspect};

/// Views of the tiling state that are built on the Windows side: the
/// `debug state` snapshot includes what Win32 reports for each managed
/// window.
pub(crate) trait TilingViews {
    /// Returns the daemon's internal workspace state, for `debug state`.
    fn snapshot(&self) -> DesktopSnapshot;
}

impl TilingViews for TilingManager {
    fn snapshot(&self) -> DesktopSnapshot {
        inspect::snapshot(self.tiler(), self.platform().self_elevated())
    }
}
