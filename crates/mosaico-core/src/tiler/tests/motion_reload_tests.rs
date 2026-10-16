//! What a config reload does to transitions.

use std::time::Duration;

use crate::config::Config;
use crate::tiler::Platform;
use crate::{Action, Direction};

use super::{animated, placements, settle, swap, two};

#[test]
fn a_reload_changes_the_transitions_that_start_after_it_not_those_in_flight() {
    // Arrange -- a swap in flight, at the default 180 ms.
    let (platform, mut manager) = two(&animated());
    swap(&mut manager);
    let in_flight = manager.frame_due(None);
    let mut slower = animated();
    slower.animations.duration = 500;

    // Act -- then the windows swap back.
    manager.reload_config(&slower);
    let after_reload = manager.frame_due(None);
    settle(&platform, &mut manager);
    let start = platform.now();
    manager.handle_action(&Action::Move(Direction::Left));
    manager.flush_pending();

    // Assert -- the reload lays the windows out where they were going.
    assert_eq!(after_reload, in_flight);
    assert_eq!(
        manager.frame_due(None),
        Some(start + Duration::from_millis(500))
    );
}

#[test]
fn a_reload_that_turns_animations_off_places_the_next_swap_at_once() {
    // Arrange
    let (platform, mut manager) = two(&animated());
    manager.reload_config(&Config::default());
    platform.take_calls();

    // Act
    swap(&mut manager);

    // Assert
    assert_eq!(placements(&platform.take_calls()).len(), 2);
    assert!(!manager.wants_frames());
}
