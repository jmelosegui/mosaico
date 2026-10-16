//! What the log is told of a motion that the user, its application or the
//! system ends: as a report says so, or as a frame finds it.

use crate::WindowEvent;
use crate::id::wid;
use crate::tiler::tests::fake_platform::FakePlatform;

use super::super::{Manager, animated, step, swap, two};
use super::starting;

/// Two windows that swap, one frame on their way, with what was logged so
/// far taken.
pub(super) fn moving() -> (FakePlatform, Manager) {
    let (platform, mut manager) = two(&animated());
    swap(&mut manager);
    step(&platform, &mut manager);
    manager.transition_lines();
    (platform, manager)
}

/// Checks that the lines logged since the last time say once that the
/// motion of window `id` ended `how`.
#[track_caller]
pub(super) fn assert_ended(manager: &mut Manager, id: u64, how: &str) {
    let lines = manager.transition_lines();
    let start = format!("transition: 0x{id} {how} after ");
    assert_eq!(starting(&lines, &start), 1, "{lines:#?}");
}

#[test]
fn a_moving_window_that_is_destroyed_is_logged_as_gone() {
    // Arrange
    let (platform, mut manager) = moving();

    // Act
    platform.close_window(1);
    manager.handle_event(&WindowEvent::Destroyed { window: wid(1) });

    // Assert
    assert_ended(&mut manager, 1, "ended, its window gone");
}

#[test]
fn a_moving_window_the_user_grabs_is_logged_as_grabbed() {
    // Arrange
    let (_platform, mut manager) = moving();

    // Act
    manager.handle_event(&WindowEvent::MoveStarted { window: wid(1) });

    // Assert
    let how = "stopped where it was, grabbed by the user";
    assert_ended(&mut manager, 1, how);
}

#[test]
fn a_moving_window_its_application_hides_is_logged_as_hidden() {
    // Arrange
    let (platform, mut manager) = moving();

    // Act
    platform.edit(1, |w| w.visible = false);
    manager.handle_event(&WindowEvent::Hidden { window: wid(1) });

    // Assert
    assert_ended(&mut manager, 1, "stopped where it was, hidden");
}

#[test]
fn a_moving_window_the_user_minimizes_is_logged_as_minimized() {
    // Arrange
    let (platform, mut manager) = moving();

    // Act
    platform.edit(1, |w| w.minimized = true);
    manager.handle_event(&WindowEvent::Minimized { window: wid(1) });

    // Assert
    assert_ended(&mut manager, 1, "stopped where it was, minimized");
}

#[test]
fn a_moving_window_a_frame_finds_maximized_is_logged_as_maximized() {
    // Arrange -- maximized, and not reported yet.
    let (platform, mut manager) = moving();
    platform.edit(1, |w| w.maximized = true);

    // Act
    step(&platform, &mut manager);

    // Assert
    assert_ended(&mut manager, 1, "stopped where it was, maximized");
}

#[test]
fn a_moving_window_a_frame_finds_minimized_is_logged_as_minimized() {
    // Arrange
    let (platform, mut manager) = moving();
    platform.edit(1, |w| w.minimized = true);

    // Act
    step(&platform, &mut manager);

    // Assert
    assert_ended(&mut manager, 1, "stopped where it was, minimized");
}

#[test]
fn moving_windows_a_frame_finds_gone_are_logged_with_their_step() {
    // Arrange -- both closed, and neither reported yet.
    let (platform, mut manager) = moving();
    platform.close_window(1);
    platform.close_window(2);

    // Act
    step(&platform, &mut manager);

    // Assert
    let lines = manager.transition_lines();
    for id in ["0x1", "0x2"] {
        let gone = format!("transition: {id} ended, its window gone after ");
        assert_eq!(starting(&lines, &gone), 1, "{lines:#?}");
    }
    let done = "transition: a step of 2 window(s) is done: ";
    assert_eq!(starting(&lines, done), 1, "{lines:#?}");
}
