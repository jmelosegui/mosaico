mod bsp;
mod three_column;
mod vertical_stack;

use serde::{Deserialize, Serialize};

use crate::Rect;

pub use bsp::BspLayout;
pub use three_column::ThreeColumnLayout;
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
/// Given a list of window handles and the available space, a layout
/// produces a position and size for each window.
pub trait Layout {
    /// Computes positions for all windows in the given work area.
    ///
    /// Returns a list of (handle, rect) pairs in the same order as the
    /// input handles.
    fn apply(&self, handles: &[usize], work_area: &Rect) -> Vec<(usize, Rect)>;
}

#[cfg(test)]
mod tests;
