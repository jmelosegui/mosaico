//! Where the edges of a tile the user resized put the splits they border.
//!
//! A split goes where its edge was dropped, as a share of the split's area,
//! to the pixel: the layout puts the edge back on the pixel it was dropped
//! at. The centered master of three columns gets the width that puts the
//! edge dropped there, and the other one mirrors it. An edge the user held
//! but hardly moved leaves its split as it is.

use crate::event::Edges;
use crate::motion::MIN_TRAVEL;
use crate::{LayoutKind, Rect};

use super::split_edges::{Border, Side, border};
use super::{MAX_SPLIT_RATIO, MIN_SPLIT_RATIO};

/// The ratio each split that an edge in `edges` of window `index` borders
/// gets once the user, who grabbed the window at `start`, dropped those
/// edges where they are in `rect`, in a layout of `kind` that has its
/// windows in `slots`, in window order, with `gap` between them: where the
/// edge is, as a share of the split's area, kept within
/// [`MIN_SPLIT_RATIO`] and [`MAX_SPLIT_RATIO`]. A corner where two splits
/// meet moves both.
///
/// An edge the drag left within [`MIN_TRAVEL`] pixels of where its slot or
/// `start` has it moves no split, as a press may wobble that much, so a
/// corner dragged along one of its edges leaves the other split as it was,
/// following the configured ratio when the user never moved it. Edges that
/// border no split are left out too, and the layout puts them back.
pub fn resized_splits(
    kind: LayoutKind,
    gap: i32,
    slots: &[Rect],
    index: usize,
    edges: Edges,
    start: Rect,
    rect: Rect,
) -> Vec<(usize, f64)> {
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
        let area = bounds(&slots[split..])?;
        let (from, len) = match side {
            Side::Left | Side::Right => (area.x, area.width),
            Side::Top | Side::Bottom => (area.y, area.height),
        };
        Some((split, ratio_at(border, at - from, len, gap)?))
    };
    grabbed
        .into_iter()
        .filter(|&(held, _)| held)
        .filter_map(|(_, side)| moved(side))
        .collect()
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

/// The ratio that puts a split a window borders as `border` where its edge
/// is, `at` from the start of the split's area, which is `len` long that
/// way, with `gap` between windows.
fn ratio_at(border: Border, at: i32, len: i32, gap: i32) -> Option<f64> {
    if len <= 0 {
        return None;
    }
    let half = gap / 2;
    // What the layout makes of the ratio: where the split is from the start
    // of its area, or how wide the centered master is.
    let size = match border {
        Border::First => at + half,
        Border::Rest => at - half,
        Border::MasterLeft => len - 2 * at,
        Border::MasterRight => 2 * at - len,
        Border::LeftColumn => len - 2 * (at + gap),
        Border::RightColumn => 2 * (at - gap) - len,
    };
    // Half a pixel more, as the layout rounds down, so that the edge lands
    // on the pixel it was dropped at.
    let ratio = (f64::from(size) + 0.5) / f64::from(len);
    Some(ratio.clamp(MIN_SPLIT_RATIO, MAX_SPLIT_RATIO))
}

/// The smallest rect that holds all of `rects`: the area of a split, from
/// the slots of the windows it divides.
fn bounds(rects: &[Rect]) -> Option<Rect> {
    let left = rects.iter().map(|r| r.x).min()?;
    let top = rects.iter().map(|r| r.y).min()?;
    let right = rects.iter().map(|r| r.x + r.width).max()?;
    let bottom = rects.iter().map(|r| r.y + r.height).max()?;
    Some(Rect::new(left, top, right - left, bottom - top))
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
