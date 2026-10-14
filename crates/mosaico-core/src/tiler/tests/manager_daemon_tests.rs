//! The tiling manager on a fake platform: what the daemon reads from
//! it, when what is held back started waiting and the state of the
//! bars.

use std::time::Duration;

use crate::Direction;
use crate::config::Config;

use super::fake_platform::FakePlatform;
use super::manager_support::start;
use super::*;

#[test]
fn what_is_held_back_is_timed_by_the_platforms_clock() {
    // Arrange -- the platform's clock is well ahead of the system's.
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    let mut manager = start(&platform, None, &Config::default());
    manager.flush_pending();
    platform.advance(Duration::from_secs(60));

    // Act -- the focus change is held back.
    manager.handle_action(&Action::Focus(Direction::Right));

    // Assert -- the scheduler counts the wait from the platform's time.
    assert_eq!(manager.held_since(), Some(platform.now()));
}

#[test]
fn every_bar_shows_the_update_notice_and_whether_hotkeys_are_paused() {
    // Arrange
    let platform = FakePlatform::new(2, &[(1, 0)]);
    let manager = start(&platform, None, &Config::default());

    // Act
    let states = manager.bar_states("v0.10.0 available", true);

    // Assert
    assert_eq!(states.len(), 2);
    assert!(
        states
            .iter()
            .all(|s| s.update_text == "v0.10.0 available" && s.paused),
        "{states:?}"
    );
}
