//! What a display change does to the windows on their way.
//!
//! As soon as the change is noted, every motion stops where its window
//! is, and each window keeps the target it was moving to. Until the change
//! is rebuilt for, placements snap: the layout they are for is about to be
//! replaced. After the rebuild, or when there is none because the monitors
//! could not be read, every window stopped that nothing placed or took out
//! of view meanwhile is placed at its target, which a rebuild that leaves
//! its monitor as it was leaves as it was too.

use crate::{Rect, WindowId};

use super::super::Platform;
use super::super::window_effects::place_and_repaint;
use super::Applier;

/// A display change the applier is waiting to see rebuilt for.
#[derive(Debug, Default)]
pub(super) struct DisplayChange {
    /// Whether a change was noted and not rebuilt for yet.
    pub(super) settling: bool,
    /// The windows it stopped, each with its target, in the order their
    /// motions started.
    stopped: Vec<(WindowId, Rect)>,
}

impl Applier {
    /// Stops every motion where its window is, as soon as a display change
    /// is noted, keeping each window's target, and has every placement snap
    /// until the change is rebuilt for.
    pub(in crate::tiler) fn display_changing(&mut self) {
        self.display.settling = true;
        let stopped = self.transitions.motions.clear();
        self.display.stopped.extend(stopped);
    }

    /// Ends a display change, rebuilt for or not: each window it stopped
    /// that nothing placed or took out of view since is placed at its
    /// target, in full.
    pub(in crate::tiler) fn display_changed(&mut self, platform: &mut dyn Platform) {
        for (window, target) in self.take_stopped() {
            place_and_repaint(window, target, platform.control());
        }
    }

    /// Ends the display change at once, and gives the windows it stopped,
    /// each with its target, for the caller to place.
    pub(super) fn take_stopped(&mut self) -> Vec<(WindowId, Rect)> {
        self.display.settling = false;
        std::mem::take(&mut self.display.stopped)
    }

    /// Forgets the target a display change kept for `window`, which is
    /// placed again, taken out of view or gone.
    pub(super) fn forget_stopped(&mut self, window: WindowId) {
        self.display.stopped.retain(|&(w, _)| w != window);
    }
}
