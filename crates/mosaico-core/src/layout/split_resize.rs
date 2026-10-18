//! Where the edges of a tile the user resized put the splits they border.
//!
//! A split goes where its edge was dropped, as a share of the split's area,
//! to the pixel: the layout puts the edge back on the pixel it was dropped
//! at. The centered master of three columns gets the width that puts the
//! edge dropped there, and the other one mirrors it. Down a stack, the edge
//! trades height between the two windows beside it alone
//! (`stack_edge.rs`). An edge the user held but hardly moved leaves its
//! split as it is. No edge goes where a window on either side of it would
//! be left below its minimum size, when [`Tiles::with_minimums`] says what
//! they are and the area has room for them.

use crate::Rect;
use crate::event::Edges;
use crate::motion::MIN_TRAVEL;

use super::heights::Heights;
use super::min_limits::limits;
use super::ratio_at::ratio_at;
use super::split_edges::{Border, Side, border};
use super::stack_edge::stack_edge;
use super::{SplitRatios, Tiles};

/// The splits of `tiles` once the user, who grabbed window `index` at
/// `start`, dropped the edges in `edges` where they are in `rect`, or
/// `None` when none of them moved a split. Each split such an edge borders
/// goes where the edge is, as a share of the split's area, kept within
/// [`MIN_SPLIT_RATIO`] and [`MAX_SPLIT_RATIO`], and down a stack the edge
/// trades height between the two windows beside it. A corner where two
/// splits meet moves both.
///
/// An edge the drag left within [`MIN_TRAVEL`] pixels of where its slot or
/// `start` has it moves no split, as a press may wobble that much, so a
/// corner dragged along one of its edges leaves the other split as it was,
/// following the configured ratio, or shared equally down a stack, when
/// the user never moved it. Edges that border no split are left out too,
/// and the layout puts them back.
pub fn resized_splits(
    tiles: Tiles<'_>,
    index: usize,
    edges: Edges,
    start: Rect,
    rect: Rect,
) -> Option<SplitRatios> {
    let Tiles {
        kind,
        gap,
        slots,
        splits,
        mins,
    } = tiles;
    let grabbed = [
        (edges.left, Side::Left),
        (edges.top, Side::Top),
        (edges.right, Side::Right),
        (edges.bottom, Side::Bottom),
    ];
    let moved = |side: Side| {
        let (split, border) = border(kind, slots.len(), index, side)?;
        let at = position(rect, side);
        let still = |other: Rect| (at - position(other, side)).abs() <= MIN_TRAVEL;
        if still(slots[index]) || still(start) {
            return None;
        }
        // Down a stack, the edge is the bottom of the window above it, a
        // whole gap above the top of the window below.
        let bottom = match border {
            Border::Above => Some(at),
            Border::Below => Some(at - gap),
            _ => None,
        };
        if let Some(bottom) = bottom {
            return Some(Moved::Heights(split, stack_edge(tiles, split, bottom)?));
        }
        let area = tiles.area_of(split)?;
        let (from, len) = match side {
            Side::Left | Side::Right => (area.x, area.width),
            Side::Top | Side::Bottom => (area.y, area.height),
        };
        let limits = limits(kind, mins, gap, split, len);
        Some(Moved::Ratio(
            split,
            ratio_at(border, at - from, len, gap, limits)?,
        ))
    };
    let moves: Vec<Moved> = grabbed
        .into_iter()
        .filter(|&(held, _)| held)
        .filter_map(|(_, side)| moved(side))
        .collect();
    if moves.is_empty() {
        return None;
    }
    let mut resized = splits.clone();
    for change in moves {
        match change {
            Moved::Ratio(split, ratio) => resized.set(kind, split, ratio),
            Moved::Heights(window, heights) => resized.set_column(kind, window, heights),
        }
    }
    Some(resized)
}

/// What a dropped edge does to the splits: a split moved to a ratio, or
/// the column of a window given new heights.
enum Moved {
    Ratio(usize, f64),
    Heights(usize, Heights),
}

/// Where the `side` edge of `rect` is.
fn position(rect: Rect, side: Side) -> i32 {
    match side {
        Side::Left => rect.x,
        Side::Right => rect.x + rect.width,
        Side::Top => rect.y,
        Side::Bottom => rect.y + rect.height,
    }
}

#[cfg(test)]
#[path = "split_resize_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "split_clamp_tests.rs"]
mod clamp_tests;

#[cfg(test)]
#[path = "split_corner_tests.rs"]
mod corner_tests;
