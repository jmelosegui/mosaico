//! The edges down a stack and down a column land where they were dropped,
//! to the pixel, whatever the gap, an odd one too. Each trades height
//! between the two windows beside it alone, from a stack nobody resized and
//! from the heights an edge dragged before left, and stops a tenth of an
//! equal share of the column from the far edge of either window.

use crate::id::wid;
use crate::{LayoutKind, Rect, WindowId};

use super::super::split_edges::Side;
use super::super::split_support::{AREA, beside, changed, dragged, edge_of, held, unmoved};
use super::super::{SplitRatios, Tiles, resized_splits};
use super::column_of;

/// Where `ratios` lay out `count` windows of `kind` with `gap` between
/// them, in window order.
fn slots(kind: LayoutKind, count: usize, gap: i32, ratios: &SplitRatios) -> Vec<Rect> {
    let windows: Vec<WindowId> = (1..=count as u64).map(wid).collect();
    let laid_out = ratios.apply(kind, gap, 0.5, &windows, &AREA);
    let slot = |window: &WindowId| laid_out.iter().find(|(w, _)| w == window).map(|w| w.1);
    windows.iter().filter_map(slot).collect()
}

/// The user drags the `side` edge of window `index` of `before`, laid out
/// with `kind` and `ratios`, `gap` apart, `by` pixels, and drops it there.
/// Returns where the window was dropped, and the splits then.
fn drag(
    (kind, gap): (LayoutKind, i32),
    before: &[Rect],
    ratios: &SplitRatios,
    (index, side): (usize, Side),
    by: i32,
) -> (Rect, SplitRatios) {
    let rect = dragged(before[index], side, by);
    let tiles = Tiles::new(kind, gap, before, ratios);
    let moved = resized_splits(tiles, index, held(side), before[index], rect);
    (rect, moved.unwrap_or_else(|| ratios.clone()))
}

#[test]
fn an_edge_down_a_stack_trades_height_between_the_two_windows_beside_it_alone() {
    let kinds = [LayoutKind::VerticalStack, LayoutKind::ThreeColumn];
    for (kind, count) in kinds.into_iter().flat_map(|k| (3..=8).map(move |c| (k, c))) {
        let edges: Vec<(usize, Side)> = (1..count)
            .flat_map(|i| [(i, Side::Top), (i, Side::Bottom)])
            .filter(|&(i, side)| beside(kind, count, i, side).is_some())
            .collect();
        for gap in 0..=9 {
            // From a stack nobody resized, and from the heights that each edge
            // dragged 23 pixels down leaves.
            let fresh = unmoved(count);
            let first = slots(kind, count, gap, &fresh);
            let once = edges
                .iter()
                .map(|&e| drag((kind, gap), &first, &fresh, e, 23).1);
            for ratios in std::iter::once(fresh.clone()).chain(once) {
                let before = slots(kind, count, gap, &ratios);
                for (&(index, side), by) in edges.iter().flat_map(|e| [(e, -37), (e, 23)]) {
                    // Act
                    let edge = (index, side);
                    let (rect, after) = drag((kind, gap), &before, &ratios, edge, by);

                    // Assert -- once laid out again.
                    let laid_out = slots(kind, count, gap, &after);
                    let case = format!("{kind:?}, {edge:?} of {count}, gap {gap}, by {by}");
                    let landed = (edge_of(laid_out[index], side), edge_of(rect, side));
                    assert_eq!(landed.0, landed.1, "{case}");
                    let two = beside(kind, count, index, side).unwrap_or_default();
                    let others = changed(&before, &laid_out);
                    assert!(others.iter().all(|w| two.contains(w)), "{case}: {others:?}");
                    // Down to the share of every other window.
                    let column = column_of(kind, count, two[0]);
                    let place = column.iter().position(|&w| w == two[0]).unwrap_or(0);
                    let (had, now) = (ratios.heights(kind, index), after.heights(kind, index));
                    let kept = |p: &usize| *p != place && *p != place + 1;
                    let same = |(had, now): (Vec<f64>, Vec<f64>)| {
                        (0..had.len()).filter(kept).all(|p| had[p] == now[p])
                    };
                    assert!(had.zip(now).is_none_or(same), "{case}");
                }
            }
        }
    }
}

#[test]
fn an_edge_down_a_stack_stops_a_tenth_of_an_equal_share_from_the_far_edge_of_either() {
    // Arrange -- windows 1 to 4 stacked right of the master, 260 pixels
    // tall each, 8 apart.
    let kind = LayoutKind::VerticalStack;
    let fresh = unmoved(5);
    let before = slots(kind, 5, 8, &fresh);

    // Act -- the bottom edge of window 1 dragged past the bottom of the
    // area, and that of window 2 past the top.
    let (_, down) = drag((kind, 8), &before, &fresh, (1, Side::Bottom), 2000);
    let (_, up) = drag((kind, 8), &before, &fresh, (2, Side::Bottom), -2000);

    // Assert -- the window that gives up the height keeps 26 pixels, a
    // tenth of the 260 an equal share of the stack has, and the others
    // keep their slots.
    let rows = |ratios: &SplitRatios| -> Vec<(i32, i32)> {
        let stack = slots(kind, 5, 8, ratios).split_off(1);
        stack.iter().map(|r| (r.y, r.height)).collect()
    };
    assert_eq!(rows(&down), [(8, 494), (510, 26), (544, 260), (812, 260)]);
    assert_eq!(rows(&up), [(8, 260), (276, 26), (310, 494), (812, 260)]);
}

#[test]
fn an_edge_down_a_column_is_worked_out_in_that_column_alone() {
    // Arrange -- seven windows in three columns: windows 1, 3 and 5 on the
    // left and 2, 4 and 6 on the right, where window 2, made short, puts
    // window 4 higher than window 3 beside it.
    let kind = LayoutKind::ThreeColumn;
    let mut ratios = unmoved(7);
    ratios.set_heights(kind, 2, &[0.2, 0.4, 0.4]);
    let before = slots(kind, 7, 8, &ratios);
    assert!(before[4].y < before[3].y);

    // Act -- the top edge of window 5, the last of the left column, 40 up.
    let (rect, after) = drag((kind, 8), &before, &ratios, (5, Side::Top), -40);

    // Assert -- it lands where it was dropped, from the top of window 3.
    assert_eq!(slots(kind, 7, 8, &after)[5].y, rect.y);
}
