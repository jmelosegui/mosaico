//! More of the edges down a stack: a window's top edge moves the split
//! above it, a corner moves the master's split with one down the stack or
//! a column, the outer edges still go back, a window that reports no grab
//! moves its split all the same, and a window dropped into a resized stack
//! from another monitor takes an equal share of it.

use crate::tiler::tests::Harness;
use crate::tiler::{BarReservation, Input};
use crate::{LayoutKind, Rect};

use super::super::super::support::{center, drag_to, swapping};
use super::super::support::{held, resize_to, resize_untold};
use super::laid_out;

#[test]
fn the_top_edge_of_a_window_of_the_stack_moves_the_split_above_it() {
    // Arrange
    let mut h = laid_out(LayoutKind::VerticalStack, 3);

    // Act -- the top edge of window 3, the last of the stack, 100 up.
    let taller = Rect::new(964, 444, 948, 628);
    resize_to(&mut h, 3, held(false, true, false, false), taller);

    // Assert -- window 2 above it gives up the height.
    assert_eq!(h.rect_of(3), taller);
    assert_eq!(h.rect_of(2), Rect::new(964, 8, 948, 428));
}

#[test]
fn a_corner_moves_the_masters_split_and_the_one_down_the_stack_or_the_column() {
    // Arrange -- the stack, and the three columns, where windows 2 and 4
    // share the left column.
    let mut stack = laid_out(LayoutKind::VerticalStack, 3);
    let mut columns = laid_out(LayoutKind::ThreeColumn, 5);

    // Act -- the top left corner of window 3 in the stack, 100 left and
    // 100 up, and the bottom right corner of window 2 in the columns, 50
    // right and 60 down.
    let stacked = Rect::new(864, 444, 1048, 628);
    resize_to(&mut stack, 3, held(true, true, false, false), stacked);
    let in_column = Rect::new(8, 8, 518, 588);
    resize_to(&mut columns, 2, held(false, false, true, true), in_column);

    // Assert -- both edges land where they were dropped.
    assert_eq!(stack.rect_of(3), stacked);
    assert_eq!(stack.rect_of(1).width, 848);
    assert_eq!(columns.rect_of(2), in_column);
    let below = columns.rect_of(4);
    assert_eq!((below.y, below.width), (604, 518));
}

#[test]
fn the_outer_edges_of_a_stack_and_of_a_column_go_back() {
    // Arrange
    let mut stack = laid_out(LayoutKind::VerticalStack, 3);
    let mut columns = laid_out(LayoutKind::ThreeColumn, 5);
    let (stacked, in_column) = (stack.rect_of(2), columns.rect_of(4));

    // Act -- the right edge of window 2 of the stack, and the left edge of
    // window 4 of the left column, both at the edge of the monitor, 40 in.
    let narrower = Rect::new(964, 8, 908, 528);
    resize_to(&mut stack, 2, held(false, false, true, false), narrower);
    let moved_in = Rect::new(in_column.x + 40, in_column.y, in_column.width - 40, 528);
    resize_to(&mut columns, 4, held(true, false, false, false), moved_in);

    // Assert
    assert_eq!((stack.rect_of(2), columns.rect_of(4)), (stacked, in_column));
}

#[test]
fn a_window_of_the_stack_that_reports_no_grab_moves_its_split_all_the_same() {
    // Arrange -- as a window that runs its own resize loop, whose grab the
    // platform cannot tell.
    let mut h = laid_out(LayoutKind::VerticalStack, 3);
    let top = h.rect_of(2);

    // Act -- its bottom edge 100 down.
    let taller = Rect {
        height: top.height + 100,
        ..top
    };
    resize_untold(&mut h, 2, held(false, false, false, true), taller);

    // Assert
    assert_eq!(h.rect_of(2), taller);
    assert_eq!(h.rect_of(3).y, taller.y + taller.height + 8);
}

#[test]
fn a_window_dropped_into_a_resized_stack_on_another_monitor_takes_an_equal_share() {
    // Arrange -- window 2 and the stack of windows 3 and 4 on the second
    // monitor, window 3 with a quarter of the stack.
    let windows = [(1, 0), (2, 1), (3, 1), (4, 1)];
    let mut h = Harness::with_settings(swapping(), 2, &windows);
    let ws = h.tiler.monitors[1].active_ws_mut();
    ws.set_layout_kind(LayoutKind::VerticalStack);
    ws.set_heights(1, &[0.25, 0.75]);
    h.step(Input::Bar(BarReservation::default()));

    // Act -- window 1 dropped on window 4.
    let over = center(h.rect_of(4));
    drag_to(&mut h, 1, over);

    // Assert -- window 1, before window 4, gets a third, which the others
    // give up in proportion.
    assert_eq!(h.active(1), vec![2, 3, 1, 4]);
    let splits = h.workspace(1, 0).split_ratios();
    let shares = splits
        .heights(LayoutKind::VerticalStack, 1)
        .unwrap_or_default();
    let thirds: Vec<f64> = shares.iter().map(|s| (s * 6.0).round()).collect();
    assert_eq!(thirds, [1.0, 2.0, 3.0]);
}
