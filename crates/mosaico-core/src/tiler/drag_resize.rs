//! What the end of a drag that resized a tiled window does to the splits.
//!
//! With `[mouse] resize_splits`, the edges a tiled window was resized by
//! move the splits they border to where the user dropped them
//! (`layout::resized_splits`): the edge between two tiles moves the split
//! between them, and a corner where two splits meet moves both. A held
//! edge the drag left within a few pixels of where it was moves none. The
//! layout then puts every window of the workspace in its slot, the resized
//! one included, so its edges that border no split, as its outer edges and
//! those between windows of a stack, go back. A window gone or minimized
//! without a report leaves the layout first, so the splits are worked out
//! against the windows that stay. A resize cancelled with Escape, a click
//! on an edge, one in monocle and one by edges that border no split are
//! handled as every resize was before: the window goes back to its slot.
//! So are a resize the tiler did not follow or that an action, from a key,
//! a command or a bar, or a display change interrupted, and one whose
//! grab the platform could not tell, which is only known to have resized.
//! `Action::TogglePause` is the daemon's, which never hands it to the
//! tiler, so pausing the hotkeys leaves a resize alone. A window left
//! maximized as the drag ends stays maximized, and moves no split.

use crate::event::{Edges, Grab};
use crate::layout::resized_splits;
use crate::{Rect, WindowId};

use super::super::{Plan, Tiler};
use super::Drag;
use super::target::cancelled;

impl Tiler {
    /// When the user resized the window `drag` is about by edges that
    /// border splits, and let go with the pointer at `pointer`, moves those
    /// splits to where the edges were dropped and lays the monitor out
    /// again. Returns whether it did.
    pub(in crate::tiler) fn move_splits(
        &mut self,
        plan: &mut Plan,
        drag: &Drag,
        pointer: Option<(i32, i32)>,
    ) -> bool {
        let Some(Grab::Edges(edges)) = drag.grab.filter(|_| self.mouse.resize_splits) else {
            return false;
        };
        // A window maximized as the drag ends stays so, as the end of any
        // drag leaves it.
        if plan.is_maximized(drag.window) {
            return false;
        }
        // The layout pass would drop a window that left without a report,
        // and the chain would shift under the new ratios.
        self.prune_stale_handles(plan, drag.monitor);
        let Some(rect) = plan.rect(drag.window) else {
            return false;
        };
        let slot = self.slot_of(drag.monitor, drag.window);
        if cancelled(drag, rect, pointer, slot) {
            return false;
        }
        let moved = self.splits_moved(drag, edges, rect);
        let Some(mon) = self
            .monitors
            .get_mut(drag.monitor)
            .filter(|_| !moved.is_empty())
        else {
            return false;
        };
        let ws = mon.active_ws_mut();
        for &(split, ratio) in &moved {
            ws.set_split_ratio(split, ratio);
        }
        crate::log_info!(
            "resize {} on mon {}: {:?}",
            drag.window,
            drag.monitor,
            moved
        );
        self.apply_layout_on(plan, drag.monitor);
        true
    }

    /// The splits that `edges` of the window `drag` is about, tiled on the
    /// workspace on screen of its monitor, border, each with the ratio it
    /// gets once the window is dropped at `rect`. None in monocle, where no
    /// edge borders a split.
    fn splits_moved(&self, drag: &Drag, edges: Edges, rect: Rect) -> Vec<(usize, f64)> {
        let idx = drag.monitor;
        let Some(ws) = self.monitors.get(idx).map(|mon| mon.active_ws()) else {
            return Vec::new();
        };
        let Some(index) = ws.index_of(drag.window).filter(|_| !ws.monocle()) else {
            return Vec::new();
        };
        let positions = self.compute_positions(idx);
        let slot = |w: &WindowId| positions.iter().find(|(p, _)| p == w).map(|&(_, r)| r);
        let slots: Vec<Rect> = ws.windows().iter().filter_map(slot).collect();
        if slots.len() != ws.len() {
            return Vec::new();
        }
        let (kind, gap) = (ws.layout_kind(), self.layout.gap);
        resized_splits(kind, gap, &slots, index, edges, drag.start, rect)
    }
}

#[cfg(test)]
#[path = "tests/drag_resize_support.rs"]
mod support;

#[cfg(test)]
#[path = "tests/drag_resize_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/drag_resize_layout_tests.rs"]
mod layout_tests;

#[cfg(test)]
#[path = "tests/drag_resize_cases_tests.rs"]
mod cases_tests;

#[cfg(test)]
#[path = "tests/drag_resize_after_tests.rs"]
mod after_tests;

#[cfg(test)]
#[path = "tests/drag_resize_motion_tests.rs"]
mod motion_tests;

#[cfg(test)]
#[path = "tests/drag_resize_held_tests.rs"]
mod held_tests;

#[cfg(test)]
#[path = "tests/drag_resize_end_tests.rs"]
mod end_tests;

#[cfg(test)]
#[path = "tests/drag_resize_bar_tests.rs"]
mod bar_tests;
