//! Asking the desktop how small a window may be.

use crate::MinSize;
use crate::WindowId;

use super::Plan;

impl Plan<'_> {
    /// The smallest size the desktop says the window may have, see
    /// [`Desktop::min_size`](super::Desktop::min_size).
    pub(in crate::tiler) fn min_size(&self, window: WindowId) -> Option<MinSize> {
        self.desktop.min_size(window)
    }
}
