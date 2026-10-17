//! Where the user moved the splits of a workspace's layouts, and the
//! heights the user gave the windows of its stacks.
//!
//! The workspace keeps one [`SplitRatios`] for all its layouts, in step
//! with its windows as `layout/split_chains.rs` says: a window added or
//! inserted gets a split that follows the default, and a share of a
//! resized stack that the others give up in proportion, and one removed
//! takes its split with it and gives its share back. In the three columns,
//! a window that comes or goes before the last of the side windows moves
//! those after it to the other column, and both columns go back to equal
//! heights. A swap leaves every split and height where it is, so the slots
//! keep their sizes and the windows trade places. Changing the layout and
//! monocle leave them alone, and the user's retile forgets them all.

#[cfg(test)]
use crate::layout::LayoutKind;
use crate::layout::{OwnSplits, SplitRatios};
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

    /// Gives the column window `window` is in, in the workspace's layout,
    /// the heights `shares`, one for each of its windows, top down, as
    /// [`SplitRatios::set_heights`] does: a seam for tests. Shares for
    /// another number of windows than the column has are left out.
    #[cfg(test)]
    pub(crate) fn set_heights(&mut self, window: usize, shares: &[f64]) {
        let sides = self.windows.len().saturating_sub(1);
        let windows = match (self.layout_kind, window.checked_sub(1)) {
            (LayoutKind::VerticalStack, Some(_)) => sides,
            (LayoutKind::ThreeColumn, Some(side)) => sides.saturating_sub(side % 2).div_ceil(2),
            _ => 0,
        };
        if shares.len() == windows {
            self.splits.set_heights(self.layout_kind, window, shares);
        }
    }

    /// Gives each window of `back`, at its index now and at the one it had
    /// when it left, what it owned there, in every layout, as
    /// [`SplitRatios::give`] does: how windows that come back get the
    /// splits and the heights they had.
    pub(crate) fn give_splits(&mut self, back: &[(usize, usize, OwnSplits)]) {
        self.splits.give(back, self.windows.len());
    }

    /// Forgets where the user moved the splits of every layout of the
    /// workspace, and the heights of its stacks, as
    /// [`SplitRatios::forget`] does: what a retile does.
    pub(crate) fn forget_split_ratios(&mut self) {
        self.splits.forget();
    }

    /// Lays out every window of the workspace in `work_area` with its
    /// layout and `gap` between windows, through its [`SplitRatios`]: each
    /// split where the user moved it, or at `ratio`, and each stack at the
    /// heights the user gave it, or shared equally.
    pub fn compute_layout(&self, gap: i32, ratio: f64, work_area: &Rect) -> Vec<(WindowId, Rect)> {
        let (kind, windows) = (self.layout_kind, &self.windows);
        self.splits.apply(kind, gap, ratio, windows, work_area)
    }
}

#[cfg(test)]
#[path = "splits_tests.rs"]
mod tests;
