//! The ratio of each split of a workspace's layouts, and the heights of the
//! windows of its stacks.
//!
//! Split `k` divides windows `k..` into window `k` and the rest. Each split
//! of the BSP chain is one, and in the vertical stack and the three columns
//! split 0, at the master's edges, is the only one. The windows of the
//! vertical stack's stack, and those of each side column of the three
//! columns, get heights of their own once the user resizes one of them
//! (`heights.rs`). A workspace keeps one set of ratios and heights per
//! layout, so cycling layouts finds each one's own again. A split the user
//! never moved follows `[layout] ratio`, and a stack nobody resized shares
//! its column equally, as every split and stack of a workspace on screen
//! does again once the user retiles.

use crate::{LayoutKind, Rect, WindowId};

use super::heights::Heights;
use super::{BspLayout, ThreeColumnLayout, VerticalStackLayout};

/// The smallest ratio a split is given, so that neither of its sides
/// vanishes: nothing tells the tiler how small a window can be. A window
/// of a resized stack keeps that share of an equal share of its column.
pub const MIN_SPLIT_RATIO: f64 = 0.1;

/// The largest ratio a split is given.
pub const MAX_SPLIT_RATIO: f64 = 0.9;

/// The ratio of each split of a workspace's layouts that the user moved,
/// the share of the split's area its first window gets, and the heights of
/// the windows of each stack the user resized.
///
/// The BSP chain has a split for each window but the last, and the stacks
/// keep a height for each of their windows, in step with the windows as
/// they come and go (`split_chains.rs`). A swap changes nothing here: the
/// slots keep their sizes and the windows trade places. The master of the
/// vertical stack and of the three columns keeps its ratio as windows come
/// and go.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SplitRatios {
    /// Each split of the BSP chain, `None` where it follows the configured
    /// ratio.
    pub(super) bsp: Vec<Option<f64>>,
    /// The master's edge in the vertical stack.
    pub(super) vertical_stack: Option<f64>,
    /// The master's edges in the three columns, where the master stays
    /// centered.
    pub(super) three_column: Option<f64>,
    /// The heights of the windows of the vertical stack's stack.
    pub(super) stack: Heights,
    /// The heights of the windows of the left column of the three columns,
    /// and of the right one.
    pub(super) columns: [Heights; 2],
}

impl SplitRatios {
    /// The ratio the user gave split `split` of `kind`, or `None` where it
    /// follows the configured ratio.
    pub fn ratio(&self, kind: LayoutKind, split: usize) -> Option<f64> {
        match kind {
            LayoutKind::Bsp => self.bsp.get(split).copied().flatten(),
            LayoutKind::VerticalStack => self.vertical_stack.filter(|_| split == 0),
            LayoutKind::ThreeColumn => self.three_column.filter(|_| split == 0),
        }
    }

    /// Moves split `split` of `kind` to `ratio`, kept within
    /// [`MIN_SPLIT_RATIO`] and [`MAX_SPLIT_RATIO`]. A split the layout does
    /// not have is left alone: in BSP, one past the chain of the windows
    /// there are, and in the other layouts, any but split 0.
    pub fn set(&mut self, kind: LayoutKind, split: usize, ratio: f64) {
        let ratio = Some(ratio.clamp(MIN_SPLIT_RATIO, MAX_SPLIT_RATIO));
        match kind {
            LayoutKind::Bsp => {
                if let Some(slot) = self.bsp.get_mut(split) {
                    *slot = ratio;
                }
            }
            LayoutKind::VerticalStack if split == 0 => self.vertical_stack = ratio,
            LayoutKind::ThreeColumn if split == 0 => self.three_column = ratio,
            _ => {}
        }
    }

    /// The share of the height of its column, without its gaps, that each
    /// window of the column window `window` of `kind` is in has, top down:
    /// the stack of the vertical stack, or the side column of the three
    /// columns. `None` for a column nobody resized, whose windows share it
    /// equally, and for the master and BSP, which have none.
    pub fn heights(&self, kind: LayoutKind, window: usize) -> Option<Vec<f64>> {
        self.column(kind, window)?.shares()
    }

    /// Gives the column window `window` of `kind` is in the heights
    /// `shares`, one for each of its windows, top down, scaled to sum to 1
    /// and each kept at [`MIN_SPLIT_RATIO`] of an equal share or more.
    pub fn set_heights(&mut self, kind: LayoutKind, window: usize, shares: &[f64]) {
        self.set_column(kind, window, Heights::of(shares));
    }

    /// The heights of the column window `window` of `kind` is in.
    pub(super) fn column(&self, kind: LayoutKind, window: usize) -> Option<&Heights> {
        match (kind, window.checked_sub(1)) {
            (LayoutKind::VerticalStack, Some(_)) => Some(&self.stack),
            (LayoutKind::ThreeColumn, Some(side)) => Some(&self.columns[side % 2]),
            _ => None,
        }
    }

    /// Gives the column window `window` of `kind` is in `heights`.
    pub(super) fn set_column(&mut self, kind: LayoutKind, window: usize, heights: Heights) {
        match (kind, window.checked_sub(1)) {
            (LayoutKind::VerticalStack, Some(_)) => self.stack = heights,
            (LayoutKind::ThreeColumn, Some(side)) => self.columns[side % 2] = heights,
            _ => {}
        }
    }

    /// Lays out `windows` in `area` with `kind` and `gap` between them, each
    /// split at its ratio here, or at `ratio` where it has none, and each
    /// stack at its heights, or shared equally.
    pub fn apply(
        &self,
        kind: LayoutKind,
        gap: i32,
        ratio: f64,
        windows: &[WindowId],
        area: &Rect,
    ) -> Vec<(WindowId, Rect)> {
        match kind {
            LayoutKind::Bsp => BspLayout { gap, ratio }.apply_splits(windows, area, &self.bsp),
            LayoutKind::VerticalStack => {
                let ratio = self.vertical_stack.unwrap_or(ratio);
                VerticalStackLayout { gap, ratio }.apply_heights(windows, area, self.stack.ends())
            }
            LayoutKind::ThreeColumn => {
                let ratio = self.three_column.unwrap_or(ratio);
                let columns = self.columns.each_ref().map(Heights::ends);
                ThreeColumnLayout { gap, ratio }.apply_heights(windows, area, columns)
            }
        }
    }
}

#[cfg(test)]
#[path = "split_ratios_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "split_chain_tests.rs"]
mod chain_tests;
