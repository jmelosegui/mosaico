//! What a reload of the config or the rules does to transitions.

use std::time::Duration;

use crate::config::WindowRule;
use crate::id::wid;
use crate::tiler::Platform;
use crate::tiler::tests::fake_platform::Call;
use crate::tiler::tests::manager_support::still;
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
    manager.reload_config(&still());
    platform.take_calls();

    // Act
    swap(&mut manager);

    // Assert
    assert_eq!(placements(&platform.take_calls()).len(), 2);
    assert!(!manager.wants_frames());
}

#[test]
fn a_moving_window_the_rules_let_go_of_has_no_border_drawn_for_it() {
    // Arrange -- window 1 grows to fill the monitor, its border hidden on
    // the way.
    let (platform, mut manager) = two(&animated());
    manager.handle_action(&Action::ToggleMonocle);
    platform.take_calls();
    platform.take_renders();
    platform.edit(1, |w| {
        w.facts.iter_mut().for_each(|f| f.title = "let go".into())
    });
    let rule = WindowRule {
        match_class: None,
        match_title: Some("let go".into()),
        match_path: None,
        manage: false,
    };

    // Act
    manager.reload_rules(vec![rule]);

    // Assert -- its border is dropped without being drawn first.
    let calls = platform.take_calls();
    let drawn = |call: &Call| matches!(call, Call::ShowBorder(window, _) if *window == wid(1));
    assert!(!calls.iter().any(drawn), "{calls:?}");
    assert!(platform.take_renders().is_empty());
}
