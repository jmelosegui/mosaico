//! What an edge dragged down a stack does to the heights of its column:
//! the edge between two windows of a stack, or of a side column, trades
//! height between those two alone, and every other edge of the column
//! stays where it was, to the pixel.
//!
//! A column nobody resized first gets heights from where its windows are,
//! so that nothing else moves. The edge then ends where it was dropped, as
//! the column rounds it, but no nearer to the far edge of either window
//! than [`MIN_SPLIT_RATIO`] of an equal share of the column, which every
//! window always keeps, so the edge never moves against the drag.

use crate::LayoutKind;

use super::Tiles;
use super::heights::Heights;

/// The windows, top down, of the column window `index` of `count` is in: the
/// stack of the vertical stack, or the side column of the three columns.
pub(super) fn column_of(kind: LayoutKind, count: usize, index: usize) -> Vec<usize> {
    match kind {
        LayoutKind::ThreeColumn if count > 2 => (2 - index % 2..count).step_by(2).collect(),
        _ => (1..count).collect(),
    }
}

/// The heights of the column of window `first` of `tiles`, which is above
/// the edge, once the edge is dropped where the bottom of `first` is at
/// `bottom`. `None` for a window with none below it in its column.
pub(super) fn stack_edge(tiles: Tiles<'_>, first: usize, bottom: i32) -> Option<Heights> {
    let Tiles {
        kind,
        gap,
        slots,
        splits,
        mins,
    } = tiles;
    let column = column_of(kind, slots.len(), first);
    let place = column.iter().position(|&i| i == first)?;
    let last = *column.last().filter(|&&last| last != first)?;
    let top = slots[column[0]].y;
    let gaps = |windows: usize| gap * windows as i32;
    let available = slots[last].y + slots[last].height - top - gaps(column.len() - 1);
    if available <= 0 {
        return None;
    }
    let share = |at: i32| f64::from(at) / f64::from(available);
    // Where each window but the last ends now, as a share of the column.
    let end = |(i, &window): (usize, &usize)| {
        let slot = slots[window];
        share(slot.y + slot.height - top - gaps(i))
    };
    let ends: Vec<f64> = match splits.column(kind, first).and_then(Heights::ends) {
        Some(ends) if ends.len() + 1 == column.len() => ends.to_vec(),
        _ => column[..column.len() - 1]
            .iter()
            .enumerate()
            .map(end)
            .collect(),
    };
    let dropped = share(bottom - top - gaps(place));
    // The two windows beside the edge keep their minimum heights.
    let least = |i: usize| share(mins.get(column[i]).map_or(0, |m| m.along(false)));
    let above = place.checked_sub(1).map_or(0.0, |i| ends[i]);
    let below = ends.get(place + 1).copied().unwrap_or(1.0);
    let (from, to) = (above + least(place), below - least(place + 1));
    let dropped = if from <= to {
        dropped.clamp(from, to)
    } else {
        dropped
    };
    Some(Heights::moved(&ends, place, dropped))
}

#[cfg(test)]
#[path = "split_stack_resize_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "split_stack_random_tests.rs"]
mod random_tests;

#[cfg(test)]
#[path = "split_stack_round_trip_tests.rs"]
mod round_trip_tests;
