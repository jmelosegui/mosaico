//! The tiler, the coalescer and the echo filter together, the way the
//! tiling manager runs them.

use crate::Direction;
use crate::config::Config;

use super::fake_platform::{Call, FakePlatform};
use super::manager_support::start;
use super::*;

#[test]
fn a_move_lands_with_its_borders_once_input_goes_quiet() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    let mut manager = start(&platform, None, &Config::default());
    let left = platform.window(1).rect;

    // Act
    manager.handle_action(&Action::Move(Direction::Right));
    manager.flush_pending();

    // Assert -- the near miss of #71: a move that stays on screen.
    assert_eq!(platform.window(2).rect, left);
    assert_eq!(platform.showing_borders(), vec![1, 2]);
}

#[test]
fn switching_workspace_right_after_a_move_leaves_no_border_behind() {
    // Arrange -- #71: the move is held until input goes quiet, and the
    // switch hides both windows before that happens.
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    let mut manager = start(&platform, None, &Config::default());

    // Act
    manager.handle_action(&Action::Move(Direction::Right));
    manager.handle_action(&Action::GoToWorkspace(2));
    manager.flush_pending();

    // Assert -- nothing is on screen, so no overlay may exist.
    assert!(
        platform.borders().is_empty(),
        "border overlays left for hidden windows: {:?}",
        platform.borders()
    );
}

#[test]
fn closing_a_window_right_after_a_move_leaves_no_border_behind() {
    // Arrange -- #71: the window is destroyed before the held move lands.
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    let mut manager = start(&platform, None, &Config::default());

    // Act
    manager.handle_action(&Action::Move(Direction::Right));
    platform.close_window(1);
    manager.handle_event(&WindowEvent::Destroyed { window: wid(1) });
    manager.flush_pending();

    // Assert
    assert!(
        !platform.borders().iter().any(|&(w, _)| w == 1),
        "border overlay left for a destroyed window: {:?}",
        platform.borders()
    );
    assert_eq!(platform.showing_borders(), vec![2]);
}

#[test]
fn a_burst_of_moves_draws_each_border_once_when_input_goes_quiet() {
    // Arrange -- #125
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0), (3, 0)]);
    let mut manager = start(&platform, None, &Config::default());
    platform.take_calls();

    // Act -- two moves of the focused window before input goes quiet.
    manager.handle_action(&Action::Move(Direction::Right));
    manager.handle_action(&Action::Move(Direction::Down));
    manager.flush_pending();

    // Assert
    let calls = platform.take_calls();
    for window in [1, 2, 3] {
        let drawn = calls
            .iter()
            .filter(|c| matches!(c, Call::ShowBorder(w, _) if *w == wid(window)))
            .count();
        assert_eq!(
            drawn, 1,
            "border of {window} drawn {drawn} times: {calls:?}"
        );
    }
}
