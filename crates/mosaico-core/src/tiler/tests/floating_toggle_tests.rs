//! Floating the focused window and tiling it again, monocle hiding the
//! floating windows, cycling through them, and the stacking that keeps them
//! above the tiles.

use crate::id::wid;
use crate::tiler::floating::place;
use crate::tiler::tests::Harness;
use crate::{Action, Direction, WindowEvent};

/// Two tiles on one monitor, window 2 floated with focus.
fn floated() -> Harness {
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    h.event(WindowEvent::Focused { window: wid(2) });
    h.action(Action::ToggleFloat);
    h
}

#[test]
fn toggling_floats_the_focused_tile_centered_at_two_thirds() {
    // Act
    let h = floated();

    // Assert
    let area = h.tiler.monitors()[0].work_area();
    assert_eq!(h.active(0), vec![1]);
    assert_eq!(h.workspace(0, 0).floating(), &[wid(2)]);
    assert_eq!(h.rect_of(2), place::first(area));
    assert!(h.rect_of(1).width > area.width / 2);
    assert_eq!(h.focus_request(), Some(2));
}

#[test]
fn toggling_again_tiles_it_and_the_next_float_goes_back_where_it_was() {
    // Arrange
    let mut h = floated();
    h.action(Action::Move(Direction::Up));
    let moved = h.rect_of(2);

    // Act
    h.action(Action::ToggleFloat);
    let tiled = h.active(0);
    h.action(Action::ToggleFloat);

    // Assert
    assert_eq!(tiled, vec![1, 2]);
    assert_eq!(h.rect_of(2), moved);
}

#[test]
fn a_tile_the_user_clicks_stays_behind_the_floating_windows() {
    // Arrange -- tiles 1 and 3 side by side, window 2 floating.
    let mut h = Harness::new(1, &[(1, 0), (2, 0), (3, 0)]);
    h.event(WindowEvent::Focused { window: wid(2) });
    h.action(Action::ToggleFloat);

    // Act
    h.event(WindowEvent::Focused { window: wid(3) });

    // Assert
    assert!(h.desktop.in_front(wid(2), wid(3)));
}

#[test]
fn tiling_a_float_again_hands_focus_to_the_next_one() {
    // Arrange -- windows 2 and 3 float, 3 in front with focus.
    let mut h = Harness::new(1, &[(1, 0), (2, 0), (3, 0)]);
    for id in [2, 3] {
        h.event(WindowEvent::Focused { window: wid(id) });
        h.action(Action::ToggleFloat);
    }

    // Act
    h.action(Action::ToggleFloat);

    // Assert
    assert_eq!(h.active(0), vec![1, 3]);
    assert_eq!(h.focus_request(), Some(2));
}

#[test]
fn monocle_hides_the_floating_windows_until_it_ends() {
    // Arrange
    let mut h = floated();
    h.action(Action::CycleFloat);
    h.event(WindowEvent::Focused { window: wid(1) });

    // Act
    h.action(Action::ToggleMonocle);
    let hidden = h.window(2).cloaked;
    h.action(Action::ToggleMonocle);

    // Assert
    assert!(hidden);
    assert!(!h.window(2).cloaked);
}

#[test]
fn monocle_does_nothing_while_a_floating_window_has_focus() {
    // Arrange
    let mut h = floated();

    // Act
    h.action(Action::ToggleMonocle);

    // Assert
    assert!(!h.workspace(0, 0).monocle());
    assert!(!h.window(2).cloaked);
}

#[test]
fn cycling_visits_every_floating_window_and_starts_at_the_front() {
    // Arrange -- windows 2 and 3 float, 3 in front, and tile 1 has focus.
    let mut h = Harness::new(1, &[(1, 0), (2, 0), (3, 0)]);
    for id in [2, 3] {
        h.event(WindowEvent::Focused { window: wid(id) });
        h.action(Action::ToggleFloat);
    }
    h.event(WindowEvent::Focused { window: wid(1) });

    // Act
    h.action(Action::CycleFloat);
    let first = h.focus_request();
    h.action(Action::CycleFloat);
    let second = h.focus_request();
    h.action(Action::CycleFloat);

    // Assert
    assert_eq!(
        (first, second, h.focus_request()),
        (Some(3), Some(2), Some(3))
    );
}

#[test]
fn a_float_that_is_tiled_again_joins_the_end_of_the_layout_with_focus() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0), (2, 0), (3, 0)]);
    h.event(WindowEvent::Focused { window: wid(1) });
    h.action(Action::ToggleFloat);

    // Act
    h.action(Action::ToggleFloat);

    // Assert
    assert_eq!(h.active(0), vec![2, 3, 1]);
    assert!(h.workspace(0, 0).floating().is_empty());
    assert_eq!(h.focus_request(), Some(1));
}
