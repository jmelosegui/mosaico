//! The tiling manager on a fake platform: what it holds back until it
//! is flushed, and the reports of its own changes it drops, timed by
//! the platform's clock.

use crate::config::{BorderColors, Config};
use crate::{Direction, WindowEvent};

use super::fake_platform::{Call, FakePlatform};
use super::manager_support::start;
use super::*;

#[test]
fn a_focus_change_waits_until_the_manager_is_flushed() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    let mut manager = start(&platform, None, &Config::default());
    manager.flush_pending();
    platform.take_calls();

    // Act
    manager.handle_action(&Action::Focus(Direction::Right));
    let held = manager.held_since().is_some();
    let before_flush = platform.take_calls();
    manager.flush_pending();

    // Assert
    assert!(held);
    assert!(
        !before_flush.contains(&Call::Focus(wid(2))),
        "{before_flush:?}"
    );
    assert!(platform.calls().contains(&Call::Focus(wid(2))));
    assert!(manager.held_since().is_none());
}

#[test]
fn a_focus_report_for_another_window_waits_for_the_held_focus_change() {
    // Arrange -- focus moves to window 2 and is held back.
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    let mut manager = start(&platform, None, &Config::default());
    manager.flush_pending();
    manager.handle_action(&Action::Focus(Direction::Right));

    // Act -- the OS hands focus back to window 1 before it lands.
    manager.handle_event(&WindowEvent::Focused { window: wid(1) });

    // Assert
    assert_eq!(manager.tiler().focused_window(), Some(wid(2)));
}

#[test]
fn nothing_held_for_a_closed_window_is_applied() {
    // Arrange -- only the focused window has a border, so window 2 has
    // none to drop when it closes, while its move is held.
    let mut config = Config::default();
    config.borders.colors.unfocused = BorderColors::NONE.into();
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    let mut manager = start(&platform, None, &config);
    manager.flush_pending();
    manager.handle_action(&Action::Move(Direction::Right));
    platform.take_calls();

    // Act
    platform.close_window(2);
    manager.handle_event(&WindowEvent::Destroyed { window: wid(2) });
    manager.flush_pending();

    // Assert
    let calls = platform.calls();
    assert!(
        !calls
            .iter()
            .any(|c| matches!(c, Call::Place(w, _) if *w == wid(2))),
        "a closed window was moved: {calls:?}"
    );
}

/// A manager that focused window 1 at startup, then window 2.
fn focused_twice(platform: &FakePlatform) -> TilingManager<FakePlatform> {
    let mut manager = start(platform, None, &Config::default());
    manager.flush_pending();
    manager.handle_action(&Action::Focus(Direction::Right));
    manager.flush_pending();
    platform.take_calls();
    manager
}

#[test]
fn a_late_report_of_an_earlier_focus_change_is_dropped() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    let mut manager = focused_twice(&platform);

    // Act -- the report of the first change arrives after the second.
    manager.handle_event(&WindowEvent::Focused { window: wid(1) });

    // Assert
    assert_eq!(manager.tiler().focused_window(), Some(wid(2)));
    assert!(platform.calls().is_empty(), "{:?}", platform.calls());
}

#[test]
fn a_focus_report_after_the_echo_has_expired_reaches_the_tiler() {
    // Arrange -- the platform's clock is the one the manager goes by.
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    let mut manager = focused_twice(&platform);
    platform.advance(FOCUS_ECHO_TTL);

    // Act
    manager.handle_event(&WindowEvent::Focused { window: wid(1) });

    // Assert
    assert_eq!(manager.tiler().focused_window(), Some(wid(1)));
}

#[test]
fn a_switch_is_timed_by_the_platforms_clock() {
    // Arrange -- the switch hides window 1, so a focus report for it is
    // taken for the switch's own until the platform's clock says
    // otherwise.
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let mut manager = start(&platform, None, &Config::default());
    manager.flush_pending();
    manager.handle_action(&Action::GoToWorkspace(2));
    manager.handle_event(&WindowEvent::Focused { window: wid(1) });
    let dropped = manager.tiler().monitors()[0].active_workspace() == 1;
    platform.advance(HIDE_FOCUS_ECHO_TTL);

    // Act
    manager.handle_event(&WindowEvent::Focused { window: wid(1) });

    // Assert -- the user picked the hidden window, so its workspace is
    // shown again.
    assert!(dropped, "the report right after the switch was followed");
    assert_eq!(manager.tiler().monitors()[0].active_workspace(), 0);
}

#[path = "manager_float_focus_tests.rs"]
mod float_tests;
