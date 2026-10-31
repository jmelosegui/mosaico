//! The keys on the focused floating window: focus leaves it for a tile,
//! move nudges it and sends it to the next monitor at the edge, and the
//! resize keys grow and shrink it around its center.

use crate::id::wid;
use crate::tiler::tests::Harness;
use crate::{Action, Direction, Rect, WindowEvent};

/// `monitors` monitors with tiles 1 and 2 on the first, window 2 floated
/// with focus.
fn floated(monitors: usize) -> Harness {
    let mut h = Harness::new(monitors, &[(1, 0), (2, 0)]);
    h.event(WindowEvent::Focused { window: wid(2) });
    h.action(Action::ToggleFloat);
    h
}

#[test]
fn moving_a_floating_window_nudges_it_and_leaves_the_tiles() {
    // Arrange
    let mut h = floated(1);
    let before = h.rect_of(2);
    let step = h.tiler.monitors()[0].work_area().width / 20;

    // Act
    h.action(Action::Move(Direction::Right));

    // Assert
    assert_eq!(
        h.rect_of(2),
        Rect {
            x: before.x + step,
            ..before
        }
    );
    assert_eq!(h.active(0), vec![1]);
}

#[test]
fn moving_a_floating_window_past_the_edge_sends_it_to_the_next_monitor() {
    // Arrange -- nudged right until it reaches the edge.
    let mut h = floated(2);
    for _ in 0..20 {
        h.action(Action::Move(Direction::Right));
    }

    // Act
    h.action(Action::Move(Direction::Right));

    // Assert
    let area = h.tiler.monitors()[1].work_area();
    assert_eq!(h.workspace(1, 0).floating(), &[wid(2)]);
    assert!(h.workspace(0, 0).floating().is_empty());
    assert_eq!(h.rect_of(2).x + h.rect_of(2).width, area.x + area.width);
    assert_eq!(h.desktop.focused, Some(wid(2)));
}

#[test]
fn the_resize_keys_grow_and_shrink_a_floating_window_around_its_center() {
    // Arrange
    let mut h = floated(1);
    let before = h.rect_of(2);

    // Act
    h.action(Action::GrowWidth);
    let wider = h.rect_of(2);
    h.action(Action::ShrinkHeight);

    // Assert
    assert!(wider.width > before.width);
    assert_eq!(wider.center_x(), before.center_x());
    assert!(h.rect_of(2).height < before.height);
}

#[test]
fn the_resize_keys_do_nothing_on_a_tile() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    h.event(WindowEvent::Focused { window: wid(1) });

    // Act
    h.action(Action::GrowWidth);

    // Assert
    assert!(h.placements().is_empty());
}

#[test]
fn focus_from_a_floating_window_goes_to_the_nearest_tile() {
    // Arrange -- tiles 1 and 3 side by side, window 2 floating.
    let mut h = Harness::new(1, &[(1, 0), (2, 0), (3, 0)]);
    h.event(WindowEvent::Focused { window: wid(2) });
    h.action(Action::ToggleFloat);

    // Act
    h.action(Action::Focus(Direction::Right));

    // Assert
    assert_eq!(h.focus_request(), Some(3));
}
