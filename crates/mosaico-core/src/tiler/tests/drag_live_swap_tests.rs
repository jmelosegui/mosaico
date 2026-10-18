//! A tiled window dragged over another tile of its own monitor, with
//! `drag_to_swap` on: that tile moves into the dragged window's slot at
//! once, as the owner asked while testing 35.6, and goes back as the
//! pointer leaves it; the drop finds it in place.

use crate::id::wid;
use crate::tiler::Timing;
use crate::tiler::drag::support::{center, drop_at, grab, report_at, swapping};
use crate::tiler::tests::Harness;
use crate::{Rect, WindowEvent};

const LEFT: Rect = Rect {
    x: 8,
    y: 8,
    width: 948,
    height: 1064,
};

const RIGHT: Rect = Rect { x: 964, ..LEFT };

/// Windows 1 and 2 side by side on one monitor, with window 1 grabbed.
fn grabbed() -> Harness {
    let mut h = Harness::with_settings(swapping(), 1, &[(1, 0), (2, 0)]);
    grab(&mut h, 1);
    h
}

#[test]
fn the_tile_under_the_pointer_takes_the_dragged_window_s_slot() {
    // Arrange
    let mut h = grabbed();

    // Act
    report_at(&mut h, 1, center(RIGHT));

    // Assert -- window 2 goes left, and nothing is committed yet.
    assert_eq!(h.placements(), vec![(2, LEFT, Timing::Now)]);
    assert_eq!(h.active(0), vec![1, 2]);
}

#[test]
fn moving_on_to_another_tile_puts_the_first_back() {
    // Arrange -- windows 2 and 3 share the right half; over window 2.
    let mut h = Harness::with_settings(swapping(), 1, &[(1, 0), (2, 0), (3, 0)]);
    let (top, bottom) = (h.rect_of(2), h.rect_of(3));
    grab(&mut h, 1);
    report_at(&mut h, 1, center(top));

    // Act -- on to window 3.
    report_at(&mut h, 1, center(bottom));

    // Assert
    let placed = vec![(2, top, Timing::Now), (3, LEFT, Timing::Now)];
    assert_eq!(h.placements(), placed);
}

#[test]
fn back_over_its_own_slot_the_tile_goes_back() {
    // Arrange
    let mut h = grabbed();
    report_at(&mut h, 1, center(RIGHT));

    // Act
    report_at(&mut h, 1, (200, 540));

    // Assert
    assert_eq!(h.placements(), vec![(2, RIGHT, Timing::Now)]);
}

#[test]
fn escape_puts_the_tile_back() {
    // Arrange -- where window 1 is before the grab.
    let mut h = Harness::with_settings(swapping(), 1, &[(1, 0), (2, 0)]);
    let start = h.rect_of(1);
    grab(&mut h, 1);
    report_at(&mut h, 1, center(RIGHT));

    // Act -- the system puts window 1 back where it started.
    h.window(1).rect = start;
    let effects = h
        .event(WindowEvent::LocationChanged { window: wid(1) })
        .to_vec();

    // Assert
    assert_eq!(h.placements(), vec![(2, RIGHT, Timing::Now)], "{effects:?}");
}

#[test]
fn the_drop_finds_the_tile_in_place_and_swaps_them() {
    // Arrange
    let mut h = grabbed();
    report_at(&mut h, 1, center(RIGHT));

    // Act
    let effects = drop_at(&mut h, 1, center(RIGHT));

    // Assert -- window 2 is placed where the drag put it, and window 1
    // takes its slot.
    assert_eq!(h.active(0), vec![2, 1]);
    let placed = vec![(2, LEFT, Timing::Now), (1, RIGHT, Timing::Now)];
    assert_eq!(h.placements(), placed, "{effects:?}");
}
