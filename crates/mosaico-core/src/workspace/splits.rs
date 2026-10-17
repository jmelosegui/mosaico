//! Where the user moved the splits of a workspace's layouts.
//!
//! The workspace keeps one [`SplitRatios`] for all its layouts, in step
//! with its windows: a window added or inserted gets a split at the
//! configured ratio, one removed takes its split with it, and a swap
//! leaves every split where it is, so the slots keep their sizes and the
//! windows trade places. Changing the layout, monocle and retiling leave
//! them alone.

use crate::layout::{LayoutKind, SplitRatios};
use crate::{Rect, WindowId};

use super::Workspace;

impl Workspace {
    /// Where the user moved the splits of each of the workspace's layouts.
    pub fn split_ratios(&self) -> &SplitRatios {
        &self.splits
    }

    /// Moves split `split` of the workspace's layout to `ratio`, as
    /// [`SplitRatios::set`] does.
    pub fn set_split_ratio(&mut self, split: usize, ratio: f64) {
        self.splits.set(self.layout_kind, split, ratio);
    }

    /// Moves split `split` of the layout `kind` to `ratio`, whichever
    /// layout is on screen, as [`SplitRatios::set`] does: how a window that
    /// comes back gets the split it had.
    pub(crate) fn set_split_ratio_of(&mut self, kind: LayoutKind, split: usize, ratio: f64) {
        self.splits.set(kind, split, ratio);
    }

    /// Lays out every window of the workspace in `work_area` with its
    /// layout and `gap` between windows, through its [`SplitRatios`]: each
    /// split where the user moved it, or at `ratio`.
    pub fn compute_layout(&self, gap: i32, ratio: f64, work_area: &Rect) -> Vec<(WindowId, Rect)> {
        let (kind, windows) = (self.layout_kind, &self.windows);
        self.splits.apply(kind, gap, ratio, windows, work_area)
    }
}

#[cfg(test)]
#[path = "splits_tests.rs"]
mod tests;
