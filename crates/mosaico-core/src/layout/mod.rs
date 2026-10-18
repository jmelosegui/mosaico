mod bsp;
mod column;
mod heights;
mod split_chains;
mod split_changes;
mod split_edges;
mod split_ratios;
mod split_resize;
mod stack_edge;
mod three_column;
mod tiles;
mod vertical_stack;

use serde::{Deserialize, Serialize};

use crate::{Rect, WindowId};

pub use bsp::BspLayout;
pub(crate) use split_chains::OwnSplits;
pub(crate) use split_changes::changes;
pub use split_ratios::{MAX_SPLIT_RATIO, MIN_SPLIT_RATIO, SplitRatios};
pub use split_resize::resized_splits;
pub use three_column::ThreeColumnLayout;
pub use tiles::Tiles;
pub use vertical_stack::VerticalStackLayout;

/// Available tiling layout algorithms.
#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LayoutKind {
    /// Binary Space Partitioning — recursive subdivision.
    #[default]
    Bsp,
    /// Master pane on the left, vertical stack on the right.
    VerticalStack,
    /// Master pane in the center, stacks on both sides.
    ThreeColumn,
}

impl LayoutKind {
    /// Returns the next layout in the cycle.
    pub fn next(self) -> Self {
        match self {
            Self::Bsp => Self::VerticalStack,
            Self::VerticalStack => Self::ThreeColumn,
            Self::ThreeColumn => Self::Bsp,
        }
    }

    /// Short display name for the status bar.
    pub fn name(self) -> &'static str {
        match self {
            Self::Bsp => "BSP",
            Self::VerticalStack => "VStack",
            Self::ThreeColumn => "3Col",
        }
    }
}

/// A layout algorithm that computes window positions within a work area.
///
/// Given a list of windows and the available space, a layout
/// produces a position and size for each window.
pub trait Layout {
    /// Computes positions for all windows in the given work area.
    ///
    /// Returns a list of (window, rect) pairs in the same order as the
    /// input windows.
    fn apply(&self, windows: &[WindowId], work_area: &Rect) -> Vec<(WindowId, Rect)>;
}

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "split_test_support.rs"]
mod split_support;

#[cfg(test)]
mod stack_reference;

#[cfg(test)]
#[path = "stack_layout_tests.rs"]
mod stack_layout_tests;
