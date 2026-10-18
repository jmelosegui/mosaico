//! Actions from the bar: they act on the bar's monitor, move focus there
//! first, and never move the pointer.

use crate::action::Origin;
use crate::config::WorkspaceMode;
use crate::id::wid;
use crate::{Action, Direction};

use super::super::Effect;
use super::super::tests::Harness;
use super::support::{both_on_second, focused, following, moves_pointer};

#[test]
fn a_click_on_the_focused_monitors_bar_switches_it_as_the_key_does() {
    // Arrange
    let mut h = both_on_second(WorkspaceMode::PerMonitor);
    assert_eq!(h.tiler.focused_monitor(), 0);

    // Act
    h.action_from(Action::GoToWorkspace(2), Origin::Bar(0));

    // Assert
    assert_eq!(h.tiler.monitors()[0].active_workspace(), 1);
    assert_eq!(h.tiler.monitors()[1].active_workspace(), 0);
    assert_eq!(h.tiler.focused_window(), Some(wid(1)));
    assert_eq!(focused(&h).last(), Some(&wid(1)));
    assert!(!moves_pointer(&h));
}

#[test]
fn a_click_on_another_monitors_bar_moves_focus_there_and_switches_it() {
    // Arrange
    let mut h = both_on_second(WorkspaceMode::PerMonitor);

    // Act
    h.action_from(Action::GoToWorkspace(2), Origin::Bar(1));

    // Assert -- monitor 0 stays on workspace 1, with window 3.
    assert_eq!(h.tiler.monitors()[1].active_workspace(), 1);
    assert_eq!(h.tiler.monitors()[0].active_workspace(), 0);
    assert_eq!(h.tiler.focused_monitor(), 1);
    assert_eq!(h.tiler.focused_window(), Some(wid(2)));
    assert_eq!(focused(&h).last(), Some(&wid(2)));
    assert!(!moves_pointer(&h));
}

#[test]
fn the_same_key_moves_the_pointer_when_it_follows_focus() {
    // Arrange -- the near miss: only the pointer's own steps leave it.
    let mut h = both_on_second(WorkspaceMode::PerMonitor);

    // Act
    h.action(Action::GoToWorkspace(2));

    // Assert
    assert!(moves_pointer(&h));
}

#[test]
fn a_click_on_the_workspace_another_monitor_shows_only_moves_focus_there() {
    // Arrange
    let mut h = both_on_second(WorkspaceMode::PerMonitor);

    // Act
    h.action_from(Action::GoToWorkspace(1), Origin::Bar(1));

    // Assert
    assert_eq!(h.tiler.monitors()[1].active_workspace(), 0);
    assert_eq!(h.tiler.focused_monitor(), 1);
    assert_eq!(h.tiler.focused_window(), Some(wid(4)));
    assert!(
        !h.effects
            .iter()
            .any(|e| matches!(e, Effect::Hide { .. } | Effect::Show { .. }))
    );
    assert!(!moves_pointer(&h));
}

#[test]
fn in_global_mode_a_click_switches_every_monitor_and_focus_goes_to_the_clicked_one() {
    // Arrange
    let settings = following(WorkspaceMode::Global);
    let mut h = Harness::with_settings(settings, 2, &[(1, 0), (2, 1), (4, 1)]);
    h.action(Action::Focus(Direction::Right));
    h.action(Action::SendToWorkspace(2));
    h.action(Action::GoToWorkspace(1));
    h.action(Action::Focus(Direction::Left));
    assert_eq!(h.tiler.focused_monitor(), 0);

    // Act
    h.action_from(Action::GoToWorkspace(2), Origin::Bar(1));

    // Assert
    assert_eq!(h.tiler.monitors()[0].active_workspace(), 1);
    assert_eq!(h.tiler.monitors()[1].active_workspace(), 1);
    assert_eq!(h.tiler.focused_monitor(), 1);
    assert_eq!(h.tiler.focused_window(), Some(wid(2)));
    assert!(!moves_pointer(&h));
}

#[test]
fn a_click_on_a_bar_the_tiler_has_no_monitor_for_does_nothing() {
    // Arrange
    let mut h = Harness::with_settings(following(WorkspaceMode::Global), 1, &[(1, 0)]);

    // Act
    let effects = h
        .action_from(Action::GoToWorkspace(2), Origin::Bar(1))
        .to_vec();

    // Assert
    assert!(effects.is_empty());
    assert_eq!(h.tiler.monitors()[0].active_workspace(), 0);
}
