//! Where a bar drew its items, and which one the pointer is over.

use crate::Rect;

/// A side of the bar, and the widget list of the config drawn there.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BarSide {
    /// The `left` list, drawn from the left edge.
    Left,
    /// The `right` list, drawn from the right edge.
    Right,
}

/// A widget of the bar config, by where its entry is: the side whose list
/// holds it, and its index in that list, counting the widgets that do not
/// show. Two widgets of the same type are told apart by it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WidgetRef {
    /// The list the widget is in.
    pub side: BarSide,
    /// Its index in that list, from 0.
    pub index: usize,
}

/// What an item drawn on a bar is, to the pointer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BarTarget {
    /// The label of a workspace: the index of the workspace, from 0.
    Workspace(usize),
    /// A widget as a whole: the container around the workspace labels,
    /// the window icon or a pill.
    Widget(WidgetRef),
}

/// Where a bar drew one of its items, and what the item is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemRegion {
    /// What the item is.
    pub target: BarTarget,
    /// Whether a click on the item does something, so the pointer shows a
    /// hand over it.
    pub clickable: bool,
    /// Where it was drawn, in bar coordinates: pixels from the bar's top
    /// left corner.
    pub rect: Rect,
}

/// The item at (`x`, `y`), in bar coordinates, among `regions` in the
/// order they were drawn. Where items overlap, the one drawn last is on
/// top, so it is the one hit. `None` over a gap between items, a
/// separator or the edges.
pub fn hit(regions: &[ItemRegion], x: i32, y: i32) -> Option<&ItemRegion> {
    regions.iter().rev().find(|region| {
        let r = &region.rect;
        x >= r.x && x < r.x + r.width && y >= r.y && y < r.y + r.height
    })
}

#[cfg(test)]
#[path = "tests/region_tests.rs"]
mod tests;
