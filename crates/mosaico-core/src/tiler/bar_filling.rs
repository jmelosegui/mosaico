//! Which windows on screen fill their monitor, so the bars are drawn
//! again as soon as one starts or stops: a bar lets the pointer through
//! over such a window (see [`Tiler::mark_maximized`]).
//!
//! Maximizing or restoring a window, and taking it in or out of
//! fullscreen, only reports that the window moved, which also comes on
//! every frame of an animation. So the tiler keeps which windows filled
//! their monitor when they last moved, and notes each change. It notes
//! all those that do as it starts, and again whenever the bars are made
//! again, so that one that filled its monitor before it was ever seen to
//! move counts too once it stops.

use std::collections::HashMap;

use crate::{MonitorId, WindowId};

use super::super::{Desktop, Plan, Tiler};
use super::fills;

/// The windows on screen that filled their monitor when they last moved,
/// with that monitor, and whether that changed since the bars were last
/// told.
#[derive(Debug, Default)]
pub(in crate::tiler) struct Filling {
    windows: HashMap<WindowId, MonitorId>,
    changed: bool,
}

impl Filling {
    /// Forgets a window that is gone.
    pub(in crate::tiler) fn forget(&mut self, window: WindowId) {
        self.windows.remove(&window);
    }
}

impl Tiler {
    /// Notes whether `window`, which moved, fills its monitor as the step
    /// leaves it: shown, not minimized, on screen as far as the tiler is
    /// concerned, and maximized or over the whole of the work area.
    pub(in crate::tiler) fn note_filling(&mut self, plan: &Plan, window: WindowId) {
        let id = plan.monitor_of(window);
        let fills_now = plan.is_visible(window)
            && !plan.is_minimized(window)
            && self.shows(window)
            && (self.monitors.iter().find(|mon| mon.id == id))
                .is_some_and(|mon| fills(mon, plan.is_maximized(window), plan.rect(window)));
        let before = if fills_now {
            self.filling.windows.insert(window, id)
        } else {
            self.filling.windows.remove(&window)
        };
        if before != fills_now.then_some(id) {
            self.filling.changed = true;
        }
    }

    /// Notes, in place of what was noted, the windows that fill their
    /// monitor as `desktop` has them now: as the tiler starts, and in the
    /// steps that go with the bars being made again, a display change and
    /// a change of the space they take. Whether that changed what was
    /// noted is left alone: the bars are drawn again after those anyway.
    pub(in crate::tiler) fn seed_filling(&mut self, desktop: &dyn Desktop) {
        let filling = self.filling_windows(desktop).into_iter();
        let windows = filling.map(|(window, at)| (window, self.monitors[at].id));
        self.filling.windows = windows.collect();
    }

    /// Whether a window on screen started or stopped filling its monitor
    /// in a step since the last call. The bars follow that with whether
    /// they let the pointer through, so they must be drawn again.
    pub fn take_bars_changed(&mut self) -> bool {
        std::mem::take(&mut self.filling.changed)
    }
}
