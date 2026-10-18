//! What the end of a drag that resized a tiled window does to the splits.
//!
//! With `[mouse] resize_splits`, the edges a tiled window was resized by
//! move the splits they border to where the user dropped them
//! (`layout::resized_splits`): the edge between two tiles moves the split
//! between them, and down a stack trades height between those two alone,
//! and a corner where two splits meet moves both. A held edge the drag left
//! within a few pixels of where it was moves none. The layout then puts
//! every window of the workspace in its slot, the resized one included, so
//! its edges that border no split, its outer edges, go back. A window gone
//! or minimized without a report leaves the layout first, so the splits are
//! worked out against the windows that stay. A resize cancelled with
//! Escape, a click on an edge, one in monocle and one by edges that border
//! no split are handled as every resize was before: the window goes back to
//! its slot. So is a resize the tiler did not follow or that an action,
//! from a key, a command or a bar, or a display change interrupted.
//! `Action::TogglePause` is the daemon's, which never hands it to the
//! tiler, so pausing the hotkeys leaves a resize alone. A window left
//! maximized as the drag ends stays maximized, and moves no split. When
//! the platform could not tell the grab, as for a window that runs its own
//! move and resize loops, the edges held are read from how the window
//! changed (`drag_resize_grab.rs`); a drag that moved both edges of an
//! axis, or left the window off its own monitor, resized nothing. While the
//! drag goes on, the tiles beyond the held edges follow them, laid out live
//! with the splits a drop then would set (`drag_live.rs`).

use crate::layout::{SplitRatios, Tiles, changes, resized_splits};
use crate::{Rect, WindowId};

use super::super::{Plan, Tiler};
use super::Drag;
use super::target::cancelled;

#[path = "drag_resize_grab.rs"]
mod grab;

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
        let Some(splits) = self.splits_dropped(plan, drag, pointer) else {
            return false;
        };
        let Some(mon) = self.monitors.get_mut(drag.monitor) else {
            return false;
        };
        let ws = mon.active_ws_mut();
        let kind = ws.layout_kind();
        let changed = changes(ws.split_ratios(), &splits, kind, ws.len());
        crate::log_info!(
            "resize {} on mon {} in {:?}: {}",
            drag.window,
            drag.monitor,
            kind,
            changed
        );
        ws.set_split_ratios(splits);
        self.apply_layout_on(plan, drag.monitor);
        true
    }

    /// The splits [`move_splits`](Self::move_splits) would set if the drag
    /// `drag` is about ended now with the pointer at `pointer`, or `None`
    /// when it would move none. A window gone or minimized without a report
    /// leaves the layout first.
    pub(in crate::tiler) fn splits_dropped(
        &mut self,
        plan: &mut Plan,
        drag: &Drag,
        pointer: Option<(i32, i32)>,
    ) -> Option<SplitRatios> {
        // A window maximized as the drag ends stays so, as the end of any
        // drag leaves it.
        if !self.mouse.resize_splits || plan.is_maximized(drag.window) {
            return None;
        }
        let edges = self.held_edges(plan, drag)?;
        // The layout pass would drop a window that left without a report,
        // and the chain would shift under the new ratios.
        self.prune_stale_handles(plan, drag.monitor);
        let rect = plan.rect(drag.window)?;
        if cancelled(drag, rect, pointer, self.slot_of(drag.monitor, drag.window)) {
            return None;
        }
        // In monocle, no edge borders a split.
        let ws = self.monitors.get(drag.monitor).map(|mon| mon.active_ws())?;
        let index = ws.index_of(drag.window).filter(|_| !ws.monocle())?;
        let positions = self.compute_positions(drag.monitor);
        let slot = |w: &WindowId| positions.iter().find(|(p, _)| p == w).map(|&(_, r)| r);
        let slots: Vec<Rect> = ws.windows().iter().filter_map(slot).collect();
        if slots.len() != ws.len() {
            return None;
        }
        let tiles = Tiles::new(ws.layout_kind(), self.layout.gap, &slots, ws.split_ratios());
        resized_splits(tiles, index, edges, drag.start, rect)
    }
}

#[cfg(test)]
#[path = "tests/drag_resize_support.rs"]
pub(super) mod support;

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

#[cfg(test)]
#[path = "tests/drag_resize_unknown_tests.rs"]
mod unknown_tests;
