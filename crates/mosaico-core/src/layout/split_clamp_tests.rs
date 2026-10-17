use super::*;

use crate::LayoutKind;

use super::super::split_support::{GAP, dragged, held, slots, unmoved};

/// What dropping the `edges` of window `index` of `slots`, laid out with
/// `kind` and grabbed in its slot, at `rect` does to the splits: each
/// split it moves, with its ratio.
fn dropped(
    kind: LayoutKind,
    slots: &[Rect],
    index: usize,
    edges: Edges,
    rect: Rect,
) -> Vec<(usize, f64)> {
    let ratios = unmoved(slots.len());
    let tiles = Tiles::new(kind, GAP, slots, &ratios);
    let moved = resized_splits(tiles, index, edges, slots[index], rect);
    let ratio = |k: usize| Some((k, moved.as_ref()?.ratio(kind, k)?));
    (0..slots.len()).filter_map(ratio).collect()
}

#[test]
fn a_split_stops_a_tenth_of_its_area_from_either_end() {
    // Arrange -- the right edge of the first of two windows dropped past
    // both ends, and the edge of a centered master past the center.
    let pair = slots(LayoutKind::Bsp, 2, &unmoved(2));
    let columns = slots(LayoutKind::ThreeColumn, 3, &unmoved(3));
    let right = held(Side::Right);
    let past = |rect: Rect, by: i32| dragged(rect, Side::Right, by);

    // Act
    let far = dropped(LayoutKind::Bsp, &pair, 0, right, past(pair[0], 5000));
    let near = dropped(LayoutKind::Bsp, &pair, 0, right, past(pair[0], -5000));
    let to_center = past(columns[0], -columns[0].width);
    let crossed = dropped(LayoutKind::ThreeColumn, &columns, 0, right, to_center);

    // Assert
    assert_eq!(far, [(0, MAX_SPLIT_RATIO)]);
    assert_eq!(near, [(0, MIN_SPLIT_RATIO)]);
    assert_eq!(crossed, [(0, MIN_SPLIT_RATIO)]);
}

#[test]
fn an_edge_that_borders_no_split_moves_none() {
    // Arrange -- the left edge of the first window, and the bottom edge of
    // the last window of the stack, both at the edge of the area.
    let bsp = slots(LayoutKind::Bsp, 3, &unmoved(3));
    let stack = slots(LayoutKind::VerticalStack, 3, &unmoved(3));
    let (left, bottom) = (held(Side::Left), held(Side::Bottom));

    // Act
    let outer = dropped(
        LayoutKind::Bsp,
        &bsp,
        0,
        left,
        dragged(bsp[0], Side::Left, 40),
    );
    let to = dragged(stack[2], Side::Bottom, -40);
    let below = dropped(LayoutKind::VerticalStack, &stack, 2, bottom, to);

    // Assert
    assert!(outer.is_empty() && below.is_empty());
}

#[test]
fn an_edge_held_but_moved_no_more_than_a_press_wobbles_moves_no_split() {
    // Arrange -- the right edge of the first of two windows, nudged up to
    // the four pixels a press may wobble, and just past them.
    let pair = slots(LayoutKind::Bsp, 2, &unmoved(2));
    let right = held(Side::Right);
    let nudged = |by: i32| dragged(pair[0], Side::Right, by);

    // Act
    let moved = [-4, -1, 1, 4, 5].map(|by| dropped(LayoutKind::Bsp, &pair, 0, right, nudged(by)));

    // Assert
    assert_eq!(moved.map(|splits| splits.len()), [0, 0, 0, 0, 1]);
}

#[test]
fn a_corner_dragged_along_one_of_its_edges_moves_only_that_split() {
    // Arrange -- in BSP, the top left corner of window 3 of three moved
    // 150 pixels left, and one down as the hand wobbles.
    let three = slots(LayoutKind::Bsp, 3, &unmoved(3));
    let corner = Edges {
        left: true,
        top: true,
        ..Edges::default()
    };
    let rect = dragged(dragged(three[2], Side::Left, -150), Side::Top, 1);

    // Act
    let moved = dropped(LayoutKind::Bsp, &three, 2, corner, rect);

    // Assert -- split 1, which its top edge borders, stays as it was.
    let splits: Vec<usize> = moved.iter().map(|&(split, _)| split).collect();
    assert_eq!(splits, [0]);
}

#[test]
fn an_edge_put_back_where_the_window_started_moves_no_split() {
    // Arrange -- the first of two windows was 200 pixels wider than its
    // slot when the user grabbed its right edge, and is put back there, as
    // Escape does.
    let pair = slots(LayoutKind::Bsp, 2, &unmoved(2));
    let start = dragged(pair[0], Side::Right, 200);

    // Act
    let right = held(Side::Right);
    let ratios = unmoved(2);
    let tiles = Tiles::new(LayoutKind::Bsp, GAP, &pair, &ratios);
    let moved = resized_splits(tiles, 0, right, start, start);

    // Assert
    assert!(moved.is_none());
}

#[test]
fn an_edge_dropped_back_on_its_slot_moves_no_split() {
    // Arrange -- the first of two windows was 200 pixels wider than its
    // slot when the user grabbed its right edge, and drops it on the edge
    // of its slot.
    let pair = slots(LayoutKind::Bsp, 2, &unmoved(2));
    let start = dragged(pair[0], Side::Right, 200);

    // Act
    let right = held(Side::Right);
    let ratios = unmoved(2);
    let tiles = Tiles::new(LayoutKind::Bsp, GAP, &pair, &ratios);
    let moved = resized_splits(tiles, 0, right, start, pair[0]);

    // Assert -- the split stays where the layout has it, following the
    // configured ratio.
    assert!(moved.is_none());
}
