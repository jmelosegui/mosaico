//! What the tiling manager tells the bars.

use crate::bar::BarState;

use super::super::{Desktop, Platform};
use super::TilingManager;

impl<P: Platform> TilingManager<P> {
    /// Returns a snapshot of bar state for each monitor, with the monitors
    /// that have a maximized or fullscreen window under their bar marked,
    /// on the desktop as the tiler sees it: a window that a hide or a
    /// minimize the tiler posted is on its way to does not count.
    pub fn bar_states(&self, update_text: &str, paused: bool) -> Vec<BarState> {
        let mut states = self.tiler.bar_states(update_text, paused);
        let now = self.platform.now();
        let mark = |desktop: &dyn Desktop| self.tiler.mark_maximized(&mut states, desktop);
        self.applier.read_view(&self.platform, now, mark);
        states
    }

    /// Whether a window on screen went in or out of maximized or
    /// fullscreen since the last call. Such a window only reports that it
    /// moved, and whether each bar lets the pointer through follows it, so
    /// the bars must be drawn again. See
    /// [`Tiler::take_bars_changed`](crate::tiler::Tiler::take_bars_changed).
    pub fn take_bars_changed(&mut self) -> bool {
        self.tiler.take_bars_changed()
    }
}
