//! Putting back the tiles a drag laid out live: where their workspace has
//! them once the drag no longer lays them out, and those a monocle layout
//! leaves where they are as another layout of their monitor runs. While
//! the tiler follows a drag, it notes where it last sent each window, put
//! back or laid out too, so the next report compares with that rather than
//! with where the desktop has a window still on its way.

use crate::{Rect, WindowId};

use super::super::super::{Plan, Tiler, Timing};
use super::Live;

impl Tiler {
    /// Puts the windows a drag moved live back where their workspace has
    /// them, once the tiler no longer follows that drag deciding where its
    /// window lands. Runs as each step ends, so a monitor the step laid out
    /// already, as a drop onto it does, is not put back twice.
    pub(in crate::tiler) fn settle_live(&mut self, plan: &mut Plan) {
        let drag = self.mouse.drag.as_ref();
        let followed = |live: &Live| drag.is_some_and(|d| d.decides && d.window == live.window);
        if let Some(live) = self.mouse.live.take_if(|live| !followed(live)) {
            self.put_back(plan, live);
        }
        if !self.mouse.drag.as_ref().is_some_and(|d| d.decides) {
            self.mouse.forget_sent();
        }
    }

    /// Notes where a layout pass sends `window`, if the drag the tiler
    /// follows sent it somewhere, so that the drag compares with that and
    /// not with where the desktop has it on the way.
    pub(in crate::tiler) fn note_laid_out(&mut self, window: WindowId, rect: Rect) {
        self.mouse.note_sent(window, rect, true);
    }

    /// Forgets what a drag laid out live without laying anything out, as the
    /// tiler stops and leaves every window where it is.
    pub(in crate::tiler) fn leave_live(&mut self) {
        self.mouse.live = None;
        self.mouse.forget_sent();
    }

    /// Forgets what a drag laid out live on monitor `idx`, as a layout of
    /// it is about to put its tiles where its workspace has them. In
    /// monocle, which places only the window in front, the others the drag
    /// moved go back where they were before.
    pub(in crate::tiler) fn forget_live(&mut self, plan: &mut Plan, idx: usize) {
        let Some(mut live) = self.mouse.live.take_if(|live| live.monitor == idx) else {
            return;
        };
        if let Some(front) = self.monocle_front(idx) {
            live.moved.retain(|&(w, ..)| w != front);
            self.put_back(plan, live);
        }
    }

    /// Puts the windows `live` moved back where the workspace on screen of
    /// its monitor has them, with a transition if due: each in its slot,
    /// or in monocle, the window in front in the monocle rect and the
    /// others where they were before the drag moved them. A window that
    /// left the workspace meanwhile is left alone, and so is one the user
    /// drags now.
    pub(super) fn put_back(&mut self, plan: &mut Plan, live: Live) {
        let idx = live.monitor;
        let Some(mon) = self.monitors.get(idx) else {
            return;
        };
        let (monitor, ws, front) = (mon.id, mon.active_ws(), self.monocle_front(idx));
        let slots = self.compute_positions(idx);
        let slot = |window: WindowId| slots.iter().find(|&&(w, _)| w == window).map(|&(_, r)| r);
        let back: Vec<(WindowId, Rect)> = live
            .moved
            .into_iter()
            .filter(|&(w, ..)| ws.contains(w) && !self.is_dragged(w))
            .filter_map(|(window, from, _)| match front {
                Some(f) if f == window => Some((window, self.monocle_rect(idx))),
                Some(_) => Some((window, from)),
                None => slot(window).map(|rect| (window, rect)),
            })
            .collect();
        for (window, rect) in back {
            plan.place(window, rect, Timing::Now, monitor);
            self.mouse.note_sent(window, rect, false);
            self.show_border_for(plan, window, rect, idx);
        }
    }
}

#[cfg(test)]
#[path = "tests/drag_live_manager_support.rs"]
mod manager_support;

#[cfg(test)]
#[path = "tests/drag_live_motion_tests.rs"]
mod motion_tests;

#[cfg(test)]
#[path = "tests/drag_live_glide_tests.rs"]
mod glide_tests;

#[cfg(test)]
#[path = "tests/drag_live_glide_drop_tests.rs"]
mod glide_drop_tests;

#[cfg(test)]
#[path = "tests/drag_live_return_tests.rs"]
mod return_tests;

#[cfg(test)]
#[path = "tests/drag_live_monocle_tests.rs"]
mod monocle_tests;

#[cfg(test)]
#[path = "tests/drag_live_monocle_drop_tests.rs"]
mod monocle_drop_tests;

#[cfg(test)]
#[path = "tests/drag_live_monocle_motion_tests.rs"]
mod monocle_motion_tests;
