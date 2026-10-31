//! Float mode: while a window floats, the tiles of its workspace are out
//! of reach of the focus and move keys, as in monocle.

use crate::id::wid;
use crate::tiler::Effect;
use crate::tiler::tests::Harness;
use crate::{Action, Direction, WindowEvent};
#[test]
fn the_tiles_are_out_of_reach_of_the_focus_keys_while_a_window_floats() {
    // Arrange -- tiles 1 and 3 side by side, window 2 floating.
    let mut h = Harness::new(1, &[(1, 0), (2, 0), (3, 0)]);
    h.event(WindowEvent::Focused { window: wid(2) });
    h.action(Action::ToggleFloat);

    // Act
    let reached: Vec<bool> = [
        Direction::Left,
        Direction::Right,
        Direction::Up,
        Direction::Down,
    ]
    .into_iter()
    .map(|dir| {
        h.action(Action::Focus(dir))
            .iter()
            .any(|e| matches!(e, Effect::Focus { .. }))
    })
    .collect();

    // Assert
    assert_eq!(reached, [false; 4]);
}

#[test]
fn focus_left_or_right_from_a_floating_window_goes_to_the_adjacent_monitor_and_back_to_it() {
    // Arrange -- window 2 floats over tile 1, tile 3 is on the second
    // monitor.
    let mut h = Harness::new(2, &[(1, 0), (2, 0), (3, 1)]);
    h.event(WindowEvent::Focused { window: wid(2) });
    h.action(Action::ToggleFloat);

    // Act
    h.action(Action::Focus(Direction::Right));
    let there = h.focus_request();
    h.action(Action::Focus(Direction::Left));

    // Assert
    assert_eq!(there, Some(3));
    assert_eq!(h.focus_request(), Some(2), "back to the floating window");
}

#[test]
fn a_tile_clicked_while_a_window_floats_does_not_reach_the_other_tiles() {
    // Arrange -- window 2 floats, and the user clicks tile 1.
    let mut h = Harness::new(1, &[(1, 0), (2, 0), (3, 0)]);
    h.event(WindowEvent::Focused { window: wid(2) });
    h.action(Action::ToggleFloat);
    h.event(WindowEvent::Focused { window: wid(1) });
    let tiles = h.active(0);

    // Act
    let focused = h.action(Action::Focus(Direction::Right)).to_vec();
    h.action(Action::Move(Direction::Right));

    // Assert
    assert!(!focused.iter().any(|e| matches!(e, Effect::Focus { .. })));
    assert_eq!(h.active(0), tiles);
}
