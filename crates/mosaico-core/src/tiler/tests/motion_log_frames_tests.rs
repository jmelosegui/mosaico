//! What the log is told as the daemon stops, and in a session without
//! frames: as the daemon cannot have display frames, the platform turns out
//! to have none, or frames fail. Without frames, every placement is marked
//! to snap, and the log hears of none on its own.

use crate::{Action, Direction};

use super::super::{animated, step, swap, two};
use super::end_tests::{assert_ended, moving};

#[test]
fn moving_windows_are_logged_as_sent_straight_to_their_targets_as_the_daemon_stops() {
    // Arrange
    let (_platform, mut manager) = moving();

    // Act
    manager.restore_all_windows();

    // Assert
    let how = "went straight to its target as the daemon stopped";
    assert_ended(&mut manager, 1, how);
}

#[test]
fn moving_windows_are_logged_as_sent_straight_to_their_targets_as_transitions_turn_off() {
    // Arrange -- as when the daemon cannot have display frames.
    let (_platform, mut manager) = moving();

    // Act
    manager.turn_off_transitions();

    // Assert
    let how = "went straight to its target, transitions turned off";
    assert_ended(&mut manager, 1, how);
}

#[test]
fn moving_windows_on_a_platform_without_frames_are_logged_as_transitions_turn_off() {
    // Arrange
    let (platform, mut manager) = moving();
    platform.refuse_frames();

    // Act
    step(&platform, &mut manager);

    // Assert
    let how = "went straight to its target, transitions turned off";
    assert_ended(&mut manager, 1, how);
}

#[test]
fn moving_windows_whose_frames_fail_are_logged_as_sent_straight_to_their_targets() {
    // Arrange
    let (platform, mut manager) = moving();
    platform.break_frames(1);

    // Act
    step(&platform, &mut manager);

    // Assert
    assert_ended(
        &mut manager,
        1,
        "went straight to its target, its frames failed",
    );
}

#[test]
fn placements_in_a_session_without_display_frames_snap_with_nothing_logged() {
    // Arrange -- as when the daemon cannot have display frames.
    let (platform, mut manager) = two(&animated());
    let (left, right) = (platform.window(1).rect, platform.window(2).rect);
    manager.turn_off_transitions();

    // Act -- a swap; then a reload of a config with transitions on, and the
    // swap undone.
    swap(&mut manager);
    let swapped = manager.transition_lines();
    manager.reload_config(&animated());
    manager.handle_action(&Action::Move(Direction::Left));
    manager.flush_pending();

    // Assert -- each placed at once, back in its slot.
    let lines = manager.transition_lines();
    assert!(swapped.is_empty(), "{swapped:#?}");
    assert!(lines.is_empty(), "{lines:#?}");
    assert_eq!([1, 2].map(|id| platform.window(id).rect), [left, right]);
    assert!(!manager.wants_frames());
}

#[test]
fn placements_on_a_platform_found_without_frames_snap_with_nothing_logged() {
    // Arrange -- the first frame of a swap finds the platform has none.
    let (platform, mut manager) = two(&animated());
    platform.refuse_frames();
    swap(&mut manager);
    step(&platform, &mut manager);
    manager.transition_lines();

    // Act -- the swap is undone.
    manager.handle_action(&Action::Move(Direction::Left));
    manager.flush_pending();

    // Assert
    let lines = manager.transition_lines();
    assert!(lines.is_empty(), "{lines:#?}");
}
