use super::*;

use super::super::split_edges::border;
use super::super::split_support::{
    GAP, KINDS, SIDES, after, dragged, edge_of, held, slots, unmoved,
};

#[test]
fn every_edge_that_borders_a_split_lands_where_it_was_dropped() {
    for kind in KINDS {
        for count in 2..=5 {
            let before = slots(kind, count, &unmoved(count));
            for (index, side) in (0..count).flat_map(|i| SIDES.map(|side| (i, side))) {
                if border(kind, count, index, side).is_none() {
                    continue;
                }
                // 17 pixels right puts the split of two windows 969 pixels
                // into its 1904, which the layout, rounding down, would put
                // a pixel short of a ratio without its half pixel more, and 5
                // is the least an edge moves for its split to follow.
                for by in [-90, -5, 17, 37] {
                    // Arrange
                    let rect = dragged(before[index], side, by);

                    // Act
                    let start = before[index];
                    let moved = resized_splits(kind, GAP, &before, index, held(side), start, rect);

                    // Assert -- to the pixel, once laid out again.
                    let laid_out = after(kind, count, &moved)[index];
                    let case = format!("{kind:?}, window {index} of {count}, {side:?} by {by}");
                    assert_eq!(moved.len(), 1, "{case}");
                    assert_eq!(edge_of(laid_out, side), edge_of(rect, side), "{case}");
                }
            }
        }
    }
}

#[test]
fn the_windows_beyond_a_split_follow_the_edge() {
    // Arrange -- in BSP, the left edge of window 3 of four, at the split
    // between window 1 and the rest, dragged 100 pixels left.
    let before = slots(LayoutKind::Bsp, 4, &unmoved(4));
    let rect = dragged(before[2], Side::Left, -100);

    // Act
    let moved = resized_splits(
        LayoutKind::Bsp,
        GAP,
        &before,
        2,
        held(Side::Left),
        before[2],
        rect,
    );

    // Assert -- window 1 gives up the width, and window 2, beyond the
    // split too, takes it with window 3.
    let laid_out = after(LayoutKind::Bsp, 4, &moved);
    assert_eq!(laid_out[0].width, before[0].width - 100);
    let second = (laid_out[1].x, laid_out[1].width);
    assert_eq!(second, (rect.x, before[1].width + 100));
    assert_eq!(laid_out[2].x, rect.x);
}

#[test]
fn a_corner_where_two_splits_meet_moves_both() {
    // Arrange -- in BSP, the top left corner of window 3 of three, the
    // bottom right one.
    let before = slots(LayoutKind::Bsp, 3, &unmoved(3));
    let corner = Edges {
        left: true,
        top: true,
        ..Edges::default()
    };
    let rect = dragged(dragged(before[2], Side::Left, -150), Side::Top, 60);

    // Act
    let moved = resized_splits(LayoutKind::Bsp, GAP, &before, 2, corner, before[2], rect);

    // Assert
    let laid_out = after(LayoutKind::Bsp, 3, &moved)[2];
    let splits: Vec<usize> = moved.iter().map(|&(split, _)| split).collect();
    assert_eq!(splits, [0, 1]);
    assert_eq!((laid_out.x, laid_out.y), (rect.x, rect.y));
}

#[test]
fn a_corner_moves_only_the_split_one_of_its_edges_borders() {
    // Arrange -- the top left corner of the second window of the stack,
    // whose top edge is between two windows of the stack.
    let before = slots(LayoutKind::VerticalStack, 3, &unmoved(3));
    let corner = Edges {
        left: true,
        top: true,
        ..Edges::default()
    };
    let rect = dragged(dragged(before[2], Side::Left, -150), Side::Top, 60);

    // Act
    let moved = resized_splits(
        LayoutKind::VerticalStack,
        GAP,
        &before,
        2,
        corner,
        before[2],
        rect,
    );

    // Assert -- the top edge goes back.
    let laid_out = after(LayoutKind::VerticalStack, 3, &moved)[2];
    assert_eq!(moved.len(), 1);
    assert_eq!((laid_out.x, laid_out.y), (rect.x, before[2].y));
}

#[test]
fn the_master_of_three_columns_stays_centered_whichever_edge_moves() {
    // Arrange -- the master's left edge dragged 100 pixels left.
    let before = slots(LayoutKind::ThreeColumn, 3, &unmoved(3));
    let rect = dragged(before[0], Side::Left, -100);

    // Act
    let moved = resized_splits(
        LayoutKind::ThreeColumn,
        GAP,
        &before,
        0,
        held(Side::Left),
        before[0],
        rect,
    );

    // Assert -- its right edge goes 100 pixels right, and both columns
    // give up the width.
    let laid_out = after(LayoutKind::ThreeColumn, 3, &moved);
    assert_eq!(laid_out[0].width, before[0].width + 200);
    assert_eq!(laid_out[0].center_x(), before[0].center_x());
    assert_eq!(laid_out[1].width, before[1].width - 100);
    assert_eq!(laid_out[2].width, before[2].width - 100);
}
