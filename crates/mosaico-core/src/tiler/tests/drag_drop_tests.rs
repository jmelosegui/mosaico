//! Where a window the user drops lands, with `drag_to_swap` on: in the
//! slot of the tile under the pointer, before it on another monitor, or
//! back in its own.

use crate::Rect;
use crate::id::wid;
use crate::tiler::tests::{Harness, MON_H, MON_W};

use super::super::support::{drag_to, swapping};

/// The center of `rect`.
fn center(rect: Rect) -> (i32, i32) {
    (rect.center_x(), rect.center_y())
}

/// Windows 1 and 2 on the first of two monitors, 3 and 4 on the second.
fn two_by_two() -> Harness {
    Harness::with_settings(swapping(), 2, &[(1, 0), (2, 0), (3, 1), (4, 1)])
}

#[test]
fn a_window_dropped_on_another_tile_swaps_places_with_it() {
    // Arrange
    let mut h = Harness::with_settings(swapping(), 1, &[(1, 0), (2, 0)]);
    let (left, right) = (h.rect_of(1), h.rect_of(2));

    // Act -- by its title bar onto window 2, with no position reported on
    // the way, as with window contents hidden while dragging.
    drag_to(&mut h, 1, center(right));

    // Assert -- it keeps focus.
    assert_eq!(h.active(0), vec![2, 1]);
    assert_eq!((h.rect_of(1), h.rect_of(2)), (right, left));
    assert_eq!(h.tiler.focused_window(), Some(wid(1)));
}

#[test]
fn a_window_dropped_on_its_own_slot_or_in_a_gap_goes_back() {
    // Arrange -- its slot, and the gap between the two tiles.
    let mut h = Harness::with_settings(swapping(), 1, &[(1, 0), (2, 0)]);
    let left = h.rect_of(1);
    for pointer in [center(left), (left.x + left.width + 4, MON_H / 2)] {
        // Act
        drag_to(&mut h, 1, pointer);

        // Assert
        assert_eq!(h.active(0), vec![1, 2], "at {pointer:?}");
        assert_eq!(h.rect_of(1), left, "at {pointer:?}");
    }
}

#[test]
fn a_window_dropped_on_another_monitor_goes_before_the_tile_under_the_pointer() {
    // Arrange
    let mut h = two_by_two();
    let bottom_right = h.rect_of(4);

    // Act
    drag_to(&mut h, 1, center(bottom_right));

    // Assert -- that monitor takes focus with it.
    assert_eq!(h.active(0), vec![2]);
    assert_eq!(h.active(1), vec![3, 1, 4]);
    assert_eq!(h.tiler.focused_monitor(), 1);
    assert!(h.rect_of(1).x >= MON_W);
    assert_eq!(h.tiler.focused_window(), Some(wid(1)));
}

#[test]
fn a_window_dropped_off_the_tiles_of_another_monitor_joins_the_end() {
    // Arrange -- the gap below its tiles.
    let mut h = two_by_two();

    // Act
    drag_to(&mut h, 1, (MON_W + MON_W / 2, MON_H - 2));

    // Assert
    assert_eq!(h.active(1), vec![3, 4, 1]);
}

#[test]
fn the_slot_a_window_gets_on_another_monitor_is_not_the_tile_it_was_dropped_on() {
    // Arrange -- in BSP, a window put before the second of two windows
    // gets the top of the right half, and the one it went before the
    // bottom.
    let mut h = two_by_two();
    let right_half = h.rect_of(4);

    // Act
    drag_to(&mut h, 1, center(right_half));

    // Assert
    let (dropped, below) = (h.rect_of(1), h.rect_of(4));
    assert_eq!(dropped.x, right_half.x);
    assert!(dropped.height < right_half.height);
    assert!(below.y > dropped.y);
}

#[path = "drag_drop_race_tests.rs"]
mod race_tests;

#[path = "drag_drop_interrupt_tests.rs"]
mod interrupt_tests;

#[path = "drag_drop_wide_tests.rs"]
mod wide_tests;
