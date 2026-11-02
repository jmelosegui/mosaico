//! The resize keys on the focused tiled window: a tile has no size of its
//! own, so they move a split it borders (`layout::stepped_splits`), as
//! dragging its edge would, and lay its monitor out again.

use crate::layout::{Axis, SplitRatios, Tiles, changes, stepped_splits};
use crate::{Rect, WindowId};

use super::floating::place;
use super::{Plan, Tiler, Timing};

impl Tiler {
    /// Grows or shrinks the focused tiled window along `axis` by one step
    /// of its work area, the step a floating window takes, by moving a
    /// split it borders. Does nothing in monocle, on a maximized window,
    /// on one on a workspace out of view, when no edge of the window along
    /// `axis` borders a split, or when the split is already as far as it
    /// goes.
    pub(super) fn resize_tile(&mut self, plan: &mut Plan, axis: Axis, grow: bool) {
        let Some(window) = self.focused_window else {
            return;
        };
        let Some((mi, wi)) = self.find_tiled(window) else {
            return;
        };
        if self.monitors[mi].active_workspace != wi || plan.is_maximized(window) {
            return;
        }
        // The layout pass would drop a window that left without a report,
        // and the chain would shift under the new ratios.
        self.prune_stale_handles(plan, mi);
        let Some(splits) = self.splits_stepped(mi, window, axis, grow) else {
            return;
        };
        let ws = self.monitors[mi].active_ws_mut();
        let kind = ws.layout_kind();
        let changed = changes(ws.split_ratios(), &splits, kind, ws.len());
        crate::log_info!("resize {window} on mon {mi} in {kind:?} by key: {changed}");
        ws.set_split_ratios(splits);
        // Held back and coalesced like a move, so a burst of presses lays
        // the monitor out once. Each press steps from the splits, not from
        // where the windows are on their way.
        self.apply_layout_positions(plan, mi, Timing::Coalesced);
        self.update_border(plan);
    }

    /// The splits of the workspace on screen of monitor `mi` once `window`
    /// grows or shrinks along `axis` by a step, or `None` when that moves
    /// no split.
    fn splits_stepped(
        &self,
        mi: usize,
        window: WindowId,
        axis: Axis,
        grow: bool,
    ) -> Option<SplitRatios> {
        let mon = self.monitors.get(mi)?;
        let ws = mon.active_ws();
        let index = ws.index_of(window).filter(|_| !ws.monocle())?;
        let slots = self.slots_in_order(mi)?;
        let mins = self.min_sizes_of(ws.windows());
        let tiles = Tiles::new(ws.layout_kind(), self.layout.gap, &slots, ws.split_ratios())
            .with_minimums(&mins);
        let step = place::step(mon.work_area, axis);
        let splits = stepped_splits(tiles, index, axis, grow, step)?;
        (&splits != ws.split_ratios()).then_some(splits)
    }

    /// The slot of every window of the workspace on screen of monitor
    /// `idx`, in window order, or `None` when the layout leaves one out.
    pub(super) fn slots_in_order(&self, idx: usize) -> Option<Vec<Rect>> {
        let ws = self.monitors.get(idx)?.active_ws();
        let positions = self.compute_positions(idx);
        let slot = |w: &WindowId| positions.iter().find(|(p, _)| p == w).map(|&(_, r)| r);
        let slots: Vec<Rect> = ws.windows().iter().filter_map(slot).collect();
        (slots.len() == ws.len()).then_some(slots)
    }
}

#[cfg(test)]
#[path = "tests/tile_keys_tests.rs"]
mod tests;
