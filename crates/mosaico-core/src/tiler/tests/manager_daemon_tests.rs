//! The tiling manager on a fake platform, as the daemon uses it: the
//! room it keeps for the bar, when what is held back started waiting,
//! and the state of the bars.

use std::time::Duration;

use crate::Direction;
use crate::config::Config;

use super::fake_platform::FakePlatform;
use super::manager_support::start;
use super::*;

#[test]
fn the_bar_keeps_its_room_on_every_monitor_through_a_display_change() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let mut manager = start(&platform, None, &Config::default());

    // Act -- a bar 30 high goes on every monitor, then a second monitor
    // is connected.
    manager.adjust_work_areas_for_bar(30, &[]);
    let reserved = manager.tiler().monitors()[0].work_area();
    manager.handle_display_change(displays(2), 30, &[]);

    // Assert -- every monitor keeps the top 30 for the bar.
    assert_eq!(reserved, Rect::new(0, 30, MON_W, MON_H - 30));
    let areas: Vec<Rect> = manager
        .tiler()
        .monitors()
        .iter()
        .map(|m| m.work_area())
        .collect();
    assert_eq!(areas.len(), 2);
    assert!(
        areas.iter().all(|a| a.y == 30 && a.height == MON_H - 30),
        "{areas:?}"
    );
}

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
