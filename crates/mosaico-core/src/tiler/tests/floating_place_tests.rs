use crate::{Direction, MinSize, Rect};

use super::*;

const AREA: Rect = Rect {
    x: 0,
    y: 0,
    width: 2000,
    height: 1000,
};

#[test]
fn a_first_float_is_centered_at_two_thirds_of_the_work_area() {
    // Act
    let rect = first(AREA);

    // Assert
    assert_eq!(rect, Rect::new(333, 167, 1333, 666));
}

#[test]
fn a_rect_larger_than_the_area_is_shrunk_and_moved_into_it() {
    // Arrange
    let rect = Rect::new(1900, -50, 2500, 400);

    // Act
    let clamped = clamp_into(rect, AREA);

    // Assert
    assert_eq!(clamped, Rect::new(0, 0, 2000, 400));
}

#[test]
fn a_move_between_monitors_keeps_the_place_relative_to_the_work_area() {
    // Arrange
    let to = Rect::new(2000, 0, 1000, 500);
    let rect = Rect::new(500, 250, 400, 200);

    // Act
    let moved = relative_move(rect, AREA, to);

    // Assert
    assert_eq!(moved, Rect::new(2250, 125, 400, 200));
}

#[test]
fn a_nudge_moves_one_twentieth_of_the_area_and_stops_at_its_edge() {
    // Arrange
    let rect = Rect::new(50, 500, 400, 200);

    // Act
    let right = nudge(rect, AREA, Direction::Right);
    let left = nudge(rect, AREA, Direction::Left);
    let down = nudge(rect, AREA, Direction::Down);

    // Assert
    assert_eq!(right.x, 150);
    assert_eq!(left.x, 0);
    assert_eq!(down.y, 550);
    assert_eq!(nudge(left, AREA, Direction::Left), left);
}

#[test]
fn a_resize_keeps_the_window_centered() {
    // Arrange
    let rect = Rect::new(800, 400, 400, 200);

    // Act
    let wider = resize(rect, AREA, Axis::Width, true, MinSize::default());
    let shorter = resize(rect, AREA, Axis::Height, false, MinSize::default());

    // Assert
    assert_eq!(wider, Rect::new(750, 400, 500, 200));
    assert_eq!(shorter, Rect::new(800, 425, 400, 150));
}

#[test]
fn a_resize_stays_within_the_area_and_the_minimum_size() {
    // Arrange
    let wide = Rect::new(0, 0, 1950, 100);
    let narrow = Rect::new(500, 0, 120, 100);

    // Act
    let grown = resize(wide, AREA, Axis::Width, true, MinSize::default());
    let shrunk = resize(narrow, AREA, Axis::Width, false, MinSize::new(100, 0));

    // Assert
    assert_eq!(grown, Rect::new(0, 0, 2000, 100));
    assert_eq!(shrunk.width, 100);
}

#[test]
fn after_a_display_change_a_float_on_another_monitor_comes_to_the_same_place_on_its_own() {
    // Arrange -- its own monitor is to the right of the one it is on.
    let own = Rect::new(2000, 0, 1000, 500);
    let rect = Rect::new(500, 250, 400, 200);

    // Act
    let settled = settle(rect, own, &[AREA, own]);

    // Assert
    assert_eq!(settled, Rect::new(2250, 125, 400, 200));
}

#[test]
fn after_a_display_change_a_float_partly_off_its_monitor_is_moved_into_it() {
    // Act
    let settled = settle(Rect::new(1700, 100, 400, 200), AREA, &[AREA]);

    // Assert
    assert_eq!(settled, Rect::new(1600, 100, 400, 200));
}

#[test]
fn after_a_display_change_a_float_on_no_monitor_is_centered_on_its_own() {
    // Act
    let settled = settle(Rect::new(-3000, 0, 400, 200), AREA, &[AREA]);

    // Assert
    assert_eq!(settled, Rect::new(800, 400, 400, 200));
}
