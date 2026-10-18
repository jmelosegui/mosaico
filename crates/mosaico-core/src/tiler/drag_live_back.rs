//! Putting back the tiles a drag laid out live: where their workspace has
//! them once the drag no longer lays them out, and those a monocle layout
//! leaves where they are as another layout of their monitor runs.

use crate::{Rect, WindowId};

use super::super::super::{Plan, Tiler, Timing};
use super::Live;

impl Tiler {
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
            self.show_border_for(plan, window, rect, idx);
        }
    }
}
