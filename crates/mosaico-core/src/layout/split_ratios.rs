//! The ratio of each split of a workspace's layouts.
//!
//! Split `k` divides windows `k..` into window `k` and the rest. Each split
//! of the BSP chain is one, and in the vertical stack and the three columns
//! split 0, at the master's edges, is the only one. A workspace keeps one set
//! of ratios per layout, so cycling layouts finds each one's own again, and a
//! split the user never moved follows `[layout] ratio`.

use crate::{LayoutKind, Rect, WindowId};

use super::{BspLayout, Layout, ThreeColumnLayout, VerticalStackLayout};

/// The smallest ratio a split is given, so that neither of its sides
/// vanishes: nothing tells the tiler how small a window can be.
pub const MIN_SPLIT_RATIO: f64 = 0.1;

/// The largest ratio a split is given.
pub const MAX_SPLIT_RATIO: f64 = 0.9;

/// The ratio of each split of a workspace's layouts that the user moved:
/// the share of the split's area its first window gets.
///
/// The BSP chain has a split for each window but the last. A window that
/// joins at `k` gets a split at the configured ratio at split `k`, and one
/// that leaves takes its own with it, so the others keep theirs. A swap
/// changes nothing here: the slots keep their sizes and the windows trade
/// places. The master of the vertical stack and of the three columns keeps
/// its ratio as windows come and go.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SplitRatios {
    /// Each split of the BSP chain, `None` where it follows the configured
    /// ratio.
    bsp: Vec<Option<f64>>,
    /// The master's edge in the vertical stack.
    vertical_stack: Option<f64>,
    /// The master's edges in the three columns, where the master stays
    /// centered.
    three_column: Option<f64>,
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

    /// Makes room for a window inserted at `index` among `count` windows:
    /// the BSP chain gets a split at the configured ratio at split `index`,
    /// or at its end. The first window gets none.
    pub(crate) fn insert(&mut self, index: usize, count: usize) {
        if count > 0 {
            let at = index.min(count - 1).min(self.bsp.len());
            self.bsp.insert(at, None);
        }
    }

    /// Forgets the split of the window removed from `index` among `count`
    /// windows: split `index` of the BSP chain, or the last one for the
    /// last window.
    pub(crate) fn remove(&mut self, index: usize, count: usize) {
        if count > 1 {
            let at = index.min(count - 2);
            if at < self.bsp.len() {
                self.bsp.remove(at);
            }
        }
    }

    /// Lays out `windows` in `area` with `kind` and `gap` between them, each
    /// split at its ratio here, or at `ratio` where it has none.
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
                VerticalStackLayout { gap, ratio }.apply(windows, area)
            }
            LayoutKind::ThreeColumn => {
                let ratio = self.three_column.unwrap_or(ratio);
                ThreeColumnLayout { gap, ratio }.apply(windows, area)
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
