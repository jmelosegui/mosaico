//! Where the resize keys put the splits a tiled window borders.
//!
//! A tile has no size of its own: the splits around it give it one. A key
//! that grows or shrinks it moves one of its edges that borders a split by
//! a step, as dragging that edge by the step would ([`resized_splits`]),
//! within the same limits: no split past [`MIN_SPLIT_RATIO`] or
//! [`MAX_SPLIT_RATIO`], and no window left below its minimum size. Its
//! right or bottom edge moves when it borders a split, and its left or top
//! one otherwise, so the last window of a row or a column grows towards
//! the others. The centered master of three columns moves both its edges by
//! half a step each, so it, too, grows by a step and stays centered.
//!
//! [`MIN_SPLIT_RATIO`]: super::MIN_SPLIT_RATIO
//! [`MAX_SPLIT_RATIO`]: super::MAX_SPLIT_RATIO

use crate::Rect;
use crate::event::Edges;
use crate::motion::MIN_TRAVEL;

use super::split_edges::{Border, Side, border};
use super::{SplitRatios, Tiles, resized_splits};

/// Which dimension of a window a resize changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    /// Its width.
    Width,
    /// Its height.
    Height,
}

/// The splits of `tiles` once window `index` grows (`grow`), or shrinks,
/// by `step` pixels along `axis`, or `None` when no edge of it along that
/// axis borders a split, as for the only window of a workspace or the
/// height of the master of a stack.
///
/// A step of [`MIN_TRAVEL`] pixels or less is taken as one more, the least
/// an edge moves for its split to follow.
pub fn stepped_splits(
    tiles: Tiles<'_>,
    index: usize,
    axis: Axis,
    grow: bool,
    step: i32,
) -> Option<SplitRatios> {
    let slot = *tiles.slots.get(index)?;
    let count = tiles.slots.len();
    let sides = match axis {
        Axis::Width => [Side::Right, Side::Left],
        Axis::Height => [Side::Bottom, Side::Top],
    };
    let (side, (_, how)) = sides
        .into_iter()
        .find_map(|side| border(tiles.kind, count, index, side).map(|b| (side, b)))?;
    let step = match how {
        Border::MasterLeft | Border::MasterRight => step / 2,
        _ => step,
    };
    let step = step.max(MIN_TRAVEL + 1);
    let outward = if grow { step } else { -step };
    resized_splits(tiles, index, held(side), slot, pushed(slot, side, outward))
}

/// The edges of a window held by its `side` edge alone.
fn held(side: Side) -> Edges {
    Edges {
        left: side == Side::Left,
        top: side == Side::Top,
        right: side == Side::Right,
        bottom: side == Side::Bottom,
    }
}

/// `rect` with its `side` edge moved `by` pixels outwards, away from its
/// center, or inwards for a negative `by`.
fn pushed(rect: Rect, side: Side, by: i32) -> Rect {
    let mut rect = rect;
    match side {
        Side::Left => (rect.x, rect.width) = (rect.x - by, rect.width + by),
        Side::Top => (rect.y, rect.height) = (rect.y - by, rect.height + by),
        Side::Right => rect.width += by,
        Side::Bottom => rect.height += by,
    }
    rect
}

#[cfg(test)]
#[path = "split_step_tests.rs"]
mod tests;
