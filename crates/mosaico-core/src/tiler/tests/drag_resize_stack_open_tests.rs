//! Windows that open in a stack the user resized get an equal share of it,
//! which every window there gives up in proportion, so none is squeezed
//! and every one stays in the work area: below an edge dragged down, below
//! an edge dragged to the bottom, below two of them, and in a side column.

use crate::event::Edges;
use crate::tiler::tests::Harness;
use crate::{LayoutKind, Rect};

use super::super::support::{held, resize_to};
use super::laid_out;

/// The bottom edge of a window, held alone.
const BOTTOM: Edges = held(false, false, false, true);

/// The user drags the bottom edge of window `id` of `h` by `by` pixels.
fn lower(h: &mut Harness, id: u64, by: i32) {
    let rect = h.rect_of(id);
    let lower = Rect {
        height: rect.height + by,
        ..rect
    };
    resize_to(h, id, BOTTOM, lower);
}

/// How tall each of the windows `ids` of `h` is.
fn heights(h: &Harness, ids: &[u64]) -> Vec<i32> {
    ids.iter().map(|&id| h.rect_of(id).height).collect()
}

/// Whether the windows `ids` of `h`, top down, fill the height of the work
/// area, from 8 to 1072, a whole gap apart, each at least a tenth of an
/// equal share of it tall.
fn fill(h: &Harness, ids: &[u64]) -> bool {
    let rects: Vec<Rect> = ids.iter().map(|&id| h.rect_of(id)).collect();
    let count = rects.len() as i32;
    let least = (1064 - 8 * (count - 1)) / count / 10;
    let apart = rects.windows(2).all(|r| r[1].y == r[0].y + r[0].height + 8);
    let end = rects.last().map(|r| r.y + r.height);
    rects[0].y == 8 && end == Some(1072) && apart && rects.iter().all(|r| r.height >= least)
}

#[test]
fn windows_that_open_below_an_edge_dragged_down_share_the_stack_in_proportion() {
    // Arrange -- windows 2 to 5 stacked right of the master, 260 pixels
    // each, the edge between windows 4 and 5 dragged 100 down.
    let mut h = laid_out(LayoutKind::VerticalStack, 5);
    lower(&mut h, 4, 100);
    let mut fourth = vec![h.rect_of(4).height];

    // Act -- windows 6, 7 and 8 open.
    for id in 6..=8 {
        h.open(id, 0);
        fourth.push(h.rect_of(4).height);
    }

    // Assert -- window 4 shrinks with each, as the others do, and windows 6
    // to 8 get a seventh of the 1016 pixels the windows share.
    assert_eq!(fourth, [360, 286, 237, 201]);
    assert_eq!(heights(&h, &[5, 6, 7, 8]), [90, 145, 145, 145]);
    assert!(fill(&h, &[2, 3, 4, 5, 6, 7, 8]));
}

#[test]
fn windows_that_open_below_an_edge_dragged_to_the_bottom_get_an_equal_share() {
    // Arrange -- the same edge dragged to the bottom of the monitor, which
    // leaves window 5 a tenth of an equal share.
    let mut h = laid_out(LayoutKind::VerticalStack, 5);
    lower(&mut h, 4, 2000);
    let pair = heights(&h, &[4, 5]);

    // Act
    for id in 6..=8 {
        h.open(id, 0);
    }

    // Assert
    assert_eq!(heights(&h, &[4, 5, 6, 7, 8]), [276, 15, 145, 145, 145]);
    assert_eq!(pair, [494, 26]);
    assert!(fill(&h, &[2, 3, 4, 5, 6, 7, 8]));
}

#[test]
fn windows_that_open_below_two_edges_dragged_to_the_bottom_stay_in_the_work_area() {
    // Arrange -- windows 2 to 6 stacked, the edges below windows 4 and 5
    // dragged to the bottom.
    let mut h = laid_out(LayoutKind::VerticalStack, 6);
    lower(&mut h, 4, 2000);
    lower(&mut h, 5, 2000);

    // Act -- window 7 opens, its top edge is dragged 50 down, and windows
    // 8 to 10 open.
    h.open(7, 0);
    let seventh = h.rect_of(7);
    let shorter = Rect::new(
        seventh.x,
        seventh.y + 50,
        seventh.width,
        seventh.height - 50,
    );
    resize_to(&mut h, 7, held(false, true, false, false), shorter);
    let stack: Vec<u64> = (2..=7).collect();
    let mut every = vec![fill(&h, &stack)];
    for id in 8..=10 {
        h.open(id, 0);
        every.push(fill(&h, &(2..=id).collect::<Vec<u64>>()));
    }

    // Assert -- every window of the stack keeps a tenth of an equal share,
    // and the stack ends at the bottom of the work area, every time.
    assert_eq!(every, [true; 4]);
}

#[test]
fn windows_that_open_in_a_resized_side_column_get_an_equal_share_of_it() {
    // Arrange -- windows 2 and 4 in the left column, the edge between them
    // dragged to the bottom.
    let mut h = laid_out(LayoutKind::ThreeColumn, 5);
    lower(&mut h, 2, 2000);
    let pair = heights(&h, &[2, 4]);

    // Act -- windows 6 and 8 join the left column, and window 7 the right.
    for id in 6..=8 {
        h.open(id, 0);
    }

    // Assert -- windows 6 and 8 get a quarter of the 1040 pixels each.
    assert_eq!(heights(&h, &[2, 4, 6, 8]), [494, 26, 260, 260]);
    assert_eq!(pair, [1003, 53]);
    assert!(fill(&h, &[2, 4, 6, 8]));
}
