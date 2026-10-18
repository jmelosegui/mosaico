//! A window dragged over another monitor goes where the slot it gets there
//! is under the pointer, so the room the tiles make opens where the user
//! holds it, as the owner asked while testing #160: over the bottom of a
//! tile as tall as the monitor, it goes after that tile, not before it.

use crate::tiler::Timing;
use crate::tiler::drag::live::support::{BOTTOM_RIGHT, TOP_RIGHT, on_two};
use crate::tiler::drag::support::{drop_at, grab, report_at, swapping};
use crate::tiler::tests::MON_W;

/// Over the top of window 4, on the right half of the second monitor.
const HIGH: (i32, i32) = (MON_W + 1438, 200);

/// Over the bottom of window 4.
const LOW: (i32, i32) = (MON_W + 1438, 800);

#[test]
fn over_the_bottom_of_a_tile_the_room_opens_under_the_pointer() {
    // Arrange
    let mut h = on_two(swapping(), &[3, 4]);
    grab(&mut h, 1);

    // Act
    report_at(&mut h, 1, LOW);

    // Assert -- window 4 moves up, leaving the bottom to window 1.
    assert_eq!(h.placements(), vec![(4, TOP_RIGHT, Timing::Now)]);
}

#[test]
fn over_the_top_of_a_tile_the_room_opens_under_the_pointer() {
    // Arrange
    let mut h = on_two(swapping(), &[3, 4]);
    grab(&mut h, 1);

    // Act
    report_at(&mut h, 1, HIGH);

    // Assert
    assert_eq!(h.placements(), vec![(4, BOTTOM_RIGHT, Timing::Now)]);
}

#[test]
fn moving_down_a_tile_moves_the_room_under_the_pointer() {
    // Arrange
    let mut h = on_two(swapping(), &[3, 4]);
    grab(&mut h, 1);
    report_at(&mut h, 1, HIGH);

    // Act
    report_at(&mut h, 1, LOW);

    // Assert
    assert_eq!(h.placements(), vec![(4, TOP_RIGHT, Timing::Now)]);
}

#[test]
fn a_drop_over_the_bottom_of_a_tile_lands_after_it() {
    // Arrange
    let mut h = on_two(swapping(), &[3, 4]);
    grab(&mut h, 1);
    report_at(&mut h, 1, LOW);

    // Act
    drop_at(&mut h, 1, LOW);

    // Assert
    assert_eq!(h.active(1), vec![3, 4, 1]);
}
