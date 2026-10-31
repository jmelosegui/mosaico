//! Holding effects back behind the ones they must follow, and dropping
//! what is held for a window that leaves.

use crate::WindowId;

use super::super::Effect;
use super::{Coalescer, waits_for};

impl Coalescer {
    /// Holds an effect until its window's placement is applied.
    ///
    /// A border takes the place of the one already held for its window,
    /// so only the latest style is drawn. Its rect may still be the slot
    /// the window is leaving, when a step that did not move the window
    /// decided it, so [`take`](Self::take) sets it from the placement.
    pub(super) fn hold_until_placed(&mut self, effect: Effect) {
        if let Effect::Border { window, .. } = effect
            && let Some(held) = self
                .after
                .iter_mut()
                .find(|e| matches!(e, Effect::Border { window: w, .. } if *w == window))
        {
            *held = effect;
            return;
        }
        self.after.push(effect);
    }

    /// Whether the held focus change is to `window`. Taking focus brings
    /// a window to the front, so restacking it has to wait for that.
    pub(super) fn focuses(&self, window: WindowId) -> bool {
        matches!(self.focus, Some(Effect::Focus { window: w, .. }) if w == window)
    }

    /// Drops everything held for `window`: its placement, what waits for
    /// it, and a focus change to it along with the restacks that wait for
    /// that. The window is gone, or leaving the screen or the layout; the
    /// tiling manager calls this when it is destroyed.
    pub fn forget(&mut self, window: WindowId) {
        self.forget_placement(window);
        if self.focuses(window) {
            self.focus = None;
            self.behind.clear();
        }
    }

    /// Drops the held placement of `window` and everything waiting for
    /// it.
    pub(super) fn forget_placement(&mut self, window: WindowId) {
        self.places.retain(|&(w, ..)| w != window);
        self.after.retain(|e| waits_for(e) != Some(window));
    }
}
