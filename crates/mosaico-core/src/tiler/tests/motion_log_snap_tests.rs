//! What the log is told of placements that snap as they are applied, for
//! what the desktop says then.

use crate::{Action, Direction};

use super::super::{animated, swap, two};
use super::starting;

/// Checks that `lines` say once that the placement of window `id` snaps
/// `why`.
#[track_caller]
fn assert_snaps(lines: &[String], id: u64, why: &str) {
    let line = format!("transition: 0x{id} snaps: {why}");
    assert_eq!(starting(lines, &line), 1, "{lines:#?}");
}

#[test]
fn a_placement_the_system_keeps_from_animating_is_logged_as_snapping_for_it() {
    // Arrange
    let (platform, mut manager) = two(&animated());
    platform.allow_animations(false);

    // Act
    swap(&mut manager);

    // Assert
    let lines = manager.transition_lines();
    assert_snaps(&lines, 1, "the system turns animations off");
}

#[test]
fn a_placement_of_a_window_minimized_since_its_step_is_logged_as_snapping_for_it() {
    // Arrange -- the swap waits for input to go quiet; meanwhile window 1
    // is minimized, and that is not reported yet.
    let (platform, mut manager) = two(&animated());
    manager.handle_action(&Action::Move(Direction::Right));
    platform.edit(1, |w| w.minimized = true);

    // Act
    manager.flush_pending();

    // Assert
    let lines = manager.transition_lines();
    assert_snaps(&lines, 1, "it is minimized");
}

#[test]
fn a_placement_of_a_window_maximized_since_its_step_is_logged_as_snapping_for_it() {
    // Arrange
    let (platform, mut manager) = two(&animated());
    manager.handle_action(&Action::Move(Direction::Right));
    platform.edit(1, |w| w.maximized = true);

    // Act
    manager.flush_pending();

    // Assert
    let lines = manager.transition_lines();
    assert_snaps(&lines, 1, "it is maximized");
}

#[test]
fn a_placement_held_as_transitions_turn_off_is_logged_as_snapping_for_it() {
    // Arrange -- the daemon finds it cannot have display frames while the
    // swap waits for input to go quiet.
    let (_platform, mut manager) = two(&animated());
    manager.handle_action(&Action::Move(Direction::Right));
    manager.turn_off_transitions();

    // Act
    manager.flush_pending();

    // Assert
    let lines = manager.transition_lines();
    assert_snaps(&lines, 1, "transitions are off for this session");
}

#[test]
fn a_placement_that_hardly_moves_its_window_is_logged_as_snapping_for_it() {
    // Arrange -- window 1 is 3 pixels from its slot.
    let (platform, mut manager) = two(&animated());
    platform.edit(1, |w| w.rect.x += 3);

    // Act
    manager.handle_action(&Action::Retile);

    // Assert
    let lines = manager.transition_lines();
    assert_snaps(&lines, 1, "it travels 3 px, 4 or less");
}
