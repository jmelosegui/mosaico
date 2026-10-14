use super::*;
use crate::id::wid;
use crate::tiler::tests::Harness;
use crate::{Action, Direction, LayoutKind};

#[test]
fn every_monitor_gets_a_bar_state_in_monitor_order() {
    // Arrange -- the first window on the first monitor starts focused.
    let h = Harness::new(2, &[(1, 0), (2, 1)]);

    // Act
    let states = h.tiler.bar_states("", false);

    // Assert
    let focused = BarState {
        focused_window: Some(wid(1)),
        ..BarState::default()
    };
    assert_eq!(states, vec![focused, BarState::default()]);
}

#[test]
fn switching_workspace_shows_on_that_monitors_bar_only() {
    // Arrange
    let mut h = Harness::new(2, &[(1, 0), (2, 1)]);

    // Act
    h.action(Action::GoToWorkspace(3));

    // Assert
    let states = h.tiler.bar_states("", false);
    assert_eq!(states[0].active_workspace, 2);
    assert_eq!(states[1].active_workspace, 0);
}

#[test]
fn cycling_the_layout_shows_the_new_layout() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0)]);

    // Act
    h.action(Action::CycleLayout);

    // Assert
    let states = h.tiler.bar_states("", false);
    assert_eq!(states[0].layout, LayoutKind::VerticalStack);
}

#[test]
fn monocle_mode_shows_on_the_bar() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);

    // Act
    h.action(Action::ToggleMonocle);

    // Assert
    let states = h.tiler.bar_states("", false);
    assert!(states[0].monocle);
}

#[test]
fn only_the_monitor_with_focus_shows_the_focused_window() {
    // Arrange
    let mut h = Harness::new(2, &[(1, 0), (2, 1)]);

    // Act
    h.action(Action::Focus(Direction::Right));

    // Assert
    let states = h.tiler.bar_states("", false);
    assert_eq!(states[0].focused_window, None);
    assert_eq!(states[1].focused_window, Some(wid(2)));
}

#[test]
fn the_update_notice_and_the_pause_reach_every_monitor() {
    // Arrange
    let h = Harness::new(2, &[(1, 0)]);

    // Act
    let states = h.tiler.bar_states("v0.10.0 available", true);

    // Assert
    assert!(
        states
            .iter()
            .all(|s| s.update_text == "v0.10.0 available" && s.paused)
    );
}
