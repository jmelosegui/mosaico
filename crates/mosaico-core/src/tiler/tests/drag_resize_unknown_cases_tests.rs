//! More drags of a window whose grab the platform cannot tell: the left
//! edge of the window on the right and a corner move their splits, while
//! a drag that moved both edges of an axis, as a move with a rescale, one
//! that left the window off its own monitor, and a move that kept its size
//! resize nothing, and end as before.

use crate::event::Edges;
use crate::tiler::Settings;
use crate::tiler::tests::Harness;
use crate::{LayoutKind, Rect};

use super::super::support::{LEFT, held, resize_untold, resizing};

/// The top edge, where the pointer holds a window that it moves.
const TOP: Edges = held(false, true, false, false);

/// Windows `ids` tiled on the second monitor, as the report has them on
/// monitor 1, with splits moved by resizes and drops deciding as the
/// defaults have them.
fn on_second(ids: &[u64]) -> Harness {
    let settings = Settings {
        drag_to_swap: true,
        ..resizing()
    };
    let windows: Vec<(u64, usize)> = ids.iter().map(|&id| (id, 1)).collect();
    Harness::with_settings(settings, 2, &windows)
}

/// The ratio of split `k` of the BSP chain on the second monitor of `h`.
fn split(h: &Harness, k: usize) -> Option<f64> {
    h.workspace(1, 0).split_ratios().ratio(LayoutKind::Bsp, k)
}

#[test]
fn the_left_edge_of_the_window_on_the_right_moves_the_split_with_no_grab_told() {
    // Arrange
    let mut h = on_second(&[1, 2]);
    let (left, right) = (h.rect_of(1), h.rect_of(2));
    let gap = right.x - (left.x + left.width);
    let wider = Rect {
        x: right.x - 300,
        width: right.width + 300,
        ..right
    };

    // Act -- window 2's left edge 300 pixels left.
    resize_untold(&mut h, 2, LEFT, wider);

    // Assert -- window 1 now ends a gap before it.
    assert_eq!(h.rect_of(2), wider);
    let first = h.rect_of(1);
    assert_eq!(first.x + first.width + gap, wider.x);
    assert!(split(&h, 0).is_some());
}

#[test]
fn a_corner_with_no_grab_told_moves_the_split_of_each_axis() {
    // Arrange -- window 2 above window 3, both right of window 1.
    let mut h = on_second(&[1, 2, 3]);
    let above = h.rect_of(2);
    let dragged = Rect {
        x: above.x - 200,
        width: above.width + 200,
        height: above.height + 100,
        ..above
    };

    // Act -- its bottom left corner 200 pixels left and 100 down.
    resize_untold(&mut h, 2, held(true, false, false, true), dragged);

    // Assert
    assert_eq!(h.rect_of(2), dragged);
    assert!(split(&h, 0).is_some() && split(&h, 1).is_some());
}

#[test]
fn a_drag_with_no_grab_told_that_moved_both_edges_of_an_axis_goes_back() {
    // Arrange -- window 1 moved 100 pixels right and made 50 wider, as the
    // system rescales a window it moves.
    let mut h = on_second(&[1, 2]);
    let left = h.rect_of(1);
    let rescaled = Rect {
        x: left.x + 100,
        width: left.width + 50,
        ..left
    };

    // Act
    resize_untold(&mut h, 1, TOP, rescaled);

    // Assert -- no resize: back in its slot, and no split moved.
    assert_eq!(h.rect_of(1), left);
    assert_eq!(split(&h, 0), None);
}

#[test]
fn a_size_change_with_no_grab_told_that_leaves_the_window_off_its_monitor_goes_back() {
    // Arrange -- window 2's left edge taken to the left end of the first
    // monitor, which puts its center there.
    let mut h = on_second(&[1, 2]);
    let right = h.rect_of(2);
    let across = Rect {
        x: 0,
        width: right.x + right.width,
        ..right
    };

    // Act
    resize_untold(&mut h, 2, LEFT, across);

    // Assert
    assert_eq!(h.active(1), vec![1, 2]);
    assert_eq!(h.rect_of(2), right);
    assert_eq!(split(&h, 0), None);
}

#[test]
fn a_move_with_no_grab_told_that_keeps_the_size_is_dropped_as_before() {
    // Arrange -- window 1 taken by its top over window 2, at its size.
    let mut h = on_second(&[1, 2]);
    let (left, right) = (h.rect_of(1), h.rect_of(2));
    let over = Rect { x: right.x, ..left };

    // Act
    resize_untold(&mut h, 1, TOP, over);

    // Assert -- the two swap, and no split moved.
    assert_eq!(h.active(1), vec![2, 1]);
    assert_eq!(split(&h, 0), None);
}
