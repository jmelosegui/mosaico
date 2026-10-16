//! Which placements the tiler lets animate, step by step.

use crate::config::AnimationConfig;
use crate::id::wid;
use crate::tiler::tests::Harness;
use crate::tiler::{Effect, Settings, Transition};
use crate::{Action, Direction, WindowEvent};

use Transition::{Animate, Snap};

#[path = "step_transition_rebuild_tests.rs"]
mod rebuild_tests;

/// Settings with transitions on.
fn animated() -> Settings {
    let animations = AnimationConfig {
        enabled: true,
        ..AnimationConfig::default()
    };
    Settings {
        animations,
        ..Settings::default()
    }
}

/// Each placement of `effects`: its window and whether it may animate.
fn marks(effects: &[Effect]) -> Vec<(u64, Transition)> {
    let mark = |e: &Effect| match *e {
        Effect::Place {
            window, transition, ..
        } => Some((window.raw(), transition)),
        _ => None,
    };
    effects.iter().filter_map(mark).collect()
}

#[test]
fn a_swap_lets_both_windows_animate() {
    // Arrange
    let mut h = Harness::with_settings(animated(), 1, &[(1, 0), (2, 0)]);

    // Act
    let effects = h.action(Action::Move(Direction::Right)).to_vec();

    // Assert
    assert_eq!(marks(&effects), [(2, Animate), (1, Animate)]);
}

#[test]
fn every_placement_snaps_while_transitions_are_off_or_take_no_time() {
    // Arrange
    let mut instant = animated();
    instant.animations.duration = 0;
    let mut off = Harness::new(1, &[(1, 0), (2, 0)]);
    let mut no_time = Harness::with_settings(instant, 1, &[(1, 0), (2, 0)]);

    // Act
    let swap = Action::Move(Direction::Right);
    let effects = [off.action(swap).to_vec(), no_time.action(swap).to_vec()];

    // Assert
    for effects in effects {
        assert_eq!(marks(&effects), [(2, Snap), (1, Snap)]);
    }
}

#[test]
fn a_window_that_opens_snaps_into_place_while_its_neighbours_make_room() {
    // Arrange
    let mut h = Harness::with_settings(animated(), 1, &[(1, 0), (2, 0)]);

    // Act
    let effects = h.open(3, 0).to_vec();

    // Assert
    assert_eq!(marks(&effects), [(1, Animate), (2, Animate), (3, Snap)]);
}

#[test]
fn the_windows_a_workspace_switch_shows_snap() {
    // Arrange
    let mut h = Harness::with_settings(animated(), 1, &[(1, 0), (2, 0)]);

    // Act -- window 1 goes to workspace 2, then window 2 comes back.
    let sent = h.action(Action::SendToWorkspace(2)).to_vec();
    let back = h.action(Action::GoToWorkspace(1)).to_vec();

    // Assert
    assert_eq!(marks(&sent), [(1, Snap)]);
    assert_eq!(marks(&back), [(2, Snap)]);
}

#[test]
fn a_window_moved_to_another_monitor_snaps_while_the_others_animate() {
    // Arrange
    let mut h = Harness::with_settings(animated(), 2, &[(1, 0), (2, 0), (3, 1)]);
    h.event(WindowEvent::Focused { window: wid(2) });

    // Act -- 1 fills its monitor, 2 enters the next on its left, 3 moves
    // over for it.
    let effects = h.action(Action::Move(Direction::Right)).to_vec();

    // Assert
    assert_eq!(marks(&effects), [(1, Animate), (2, Snap), (3, Animate)]);
}

#[test]
fn monocle_lets_the_window_that_fills_the_screen_animate_both_ways() {
    // Arrange
    let mut h = Harness::with_settings(animated(), 1, &[(1, 0), (2, 0)]);

    // Act
    let on = h.action(Action::ToggleMonocle).to_vec();
    let off = h.action(Action::ToggleMonocle).to_vec();

    // Assert -- window 2 is not placed until monocle goes off.
    assert_eq!(marks(&on), [(1, Animate)]);
    assert_eq!(marks(&off), [(1, Animate), (2, Animate)]);
}
