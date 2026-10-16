//! The tiler, the coalescer and the echo filter together, the way the
//! tiling manager runs them. Each test runs with transitions off, and
//! again with them on, once the windows have settled.

use crate::Direction;
use crate::config::Config;

use super::fake_platform::{Call, FakePlatform};
use super::manager_support::{animated, settle, start, still};
use super::*;

fn a_move_lands_with_its_borders(config: &Config) {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    let mut manager = start(&platform, None, config);
    let left = platform.window(1).rect;

    // Act
    manager.handle_action(&Action::Move(Direction::Right));
    manager.flush_pending();
    settle(&platform, &mut manager);

    // Assert -- the near miss of #71: a move that stays on screen.
    assert_eq!(platform.window(2).rect, left);
    assert_eq!(platform.showing_borders(), vec![1, 2]);
}

#[test]
fn a_move_lands_with_its_borders_once_input_goes_quiet() {
    a_move_lands_with_its_borders(&still());
}

#[test]
fn a_move_lands_with_its_borders_once_input_goes_quiet_with_transitions_on() {
    a_move_lands_with_its_borders(&animated());
}

fn a_switch_right_after_a_move_leaves_no_border(config: &Config) {
    // Arrange -- #71: the move is held until input goes quiet, and the
    // switch hides both windows before that happens.
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    let mut manager = start(&platform, None, config);

    // Act
    manager.handle_action(&Action::Move(Direction::Right));
    manager.handle_action(&Action::GoToWorkspace(2));
    manager.flush_pending();
    settle(&platform, &mut manager);

    // Assert -- nothing is on screen, so no overlay may exist.
    let borders = platform.borders();
    assert!(
        borders.is_empty(),
        "overlays left for hidden windows: {borders:?}"
    );
}

#[test]
fn switching_workspace_right_after_a_move_leaves_no_border_behind() {
    a_switch_right_after_a_move_leaves_no_border(&still());
}

#[test]
fn switching_workspace_right_after_a_move_leaves_no_border_behind_with_transitions_on() {
    a_switch_right_after_a_move_leaves_no_border(&animated());
}

fn a_close_right_after_a_move_leaves_no_border(config: &Config) {
    // Arrange -- #71: the window is destroyed before the held move lands.
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    let mut manager = start(&platform, None, config);

    // Act
    manager.handle_action(&Action::Move(Direction::Right));
    platform.close_window(1);
    manager.handle_event(&WindowEvent::Destroyed { window: wid(1) });
    manager.flush_pending();
    settle(&platform, &mut manager);

    // Assert
    let borders = platform.borders();
    let left = borders.iter().any(|&(w, _)| w == 1);
    assert!(!left, "overlay left for a destroyed window: {borders:?}");
    assert_eq!(platform.showing_borders(), vec![2]);
}

#[test]
fn closing_a_window_right_after_a_move_leaves_no_border_behind() {
    a_close_right_after_a_move_leaves_no_border(&still());
}

#[test]
fn closing_a_window_right_after_a_move_leaves_no_border_behind_with_transitions_on() {
    a_close_right_after_a_move_leaves_no_border(&animated());
}

fn a_burst_of_moves_draws_each_border_once(config: &Config) {
    // Arrange -- #125
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0), (3, 0)]);
    let mut manager = start(&platform, None, config);
    platform.take_calls();

    // Act -- two moves of the focused window before input goes quiet.
    manager.handle_action(&Action::Move(Direction::Right));
    manager.handle_action(&Action::Move(Direction::Down));
    manager.flush_pending();
    settle(&platform, &mut manager);

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

#[test]
fn a_burst_of_moves_draws_each_border_once_when_input_goes_quiet() {
    a_burst_of_moves_draws_each_border_once(&still());
}

#[test]
fn a_burst_of_moves_draws_each_border_once_when_input_goes_quiet_with_transitions_on() {
    a_burst_of_moves_draws_each_border_once(&animated());
}
