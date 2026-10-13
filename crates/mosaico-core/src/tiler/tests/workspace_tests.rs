use crate::config::{HidingBehaviour, WorkspaceMode};

use super::*;

fn with_mode(mode: WorkspaceMode, hiding: HidingBehaviour) -> Settings {
    let mut settings = Settings::default();
    settings.workspaces.mode = mode;
    settings.layout.hiding = hiding;
    settings
}

#[test]
fn going_to_a_workspace_shows_it_before_hiding_the_one_left() {
    // Arrange -- window 1 on workspace 2 (on screen), window 2 on 1.
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    h.action(Action::SendToWorkspace(2));

    // Act
    h.action(Action::GoToWorkspace(1));

    // Assert
    let show = h.effects.iter().position(|e| {
        *e == Effect::Show {
            window: 2,
            how: HidingBehaviour::Cloak,
        }
    });
    let hide = h.effects.iter().position(|e| {
        *e == Effect::Hide {
            window: 1,
            how: HidingBehaviour::Cloak,
        }
    });
    assert!(show.is_some() && hide.is_some());
    assert!(
        show < hide,
        "the window leaving keeps focus until the target is up"
    );
    assert_eq!(h.tiler.monitors()[0].active_workspace(), 0);
}

#[test]
fn switching_back_restores_focus_to_the_remembered_window() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    h.event(WindowEvent::MouseHover { hwnd: 2 });
    assert_eq!(h.tiler.focused_window(), Some(2));

    // Act
    h.action(Action::GoToWorkspace(3));
    h.action(Action::GoToWorkspace(1));

    // Assert
    assert_eq!(h.tiler.focused_window(), Some(2));
    assert_eq!(h.focus_request(), Some(2));
    let first_show = h
        .effects
        .iter()
        .position(|e| matches!(e, Effect::Show { .. }));
    let first_focus = h
        .effects
        .iter()
        .position(|e| matches!(e, Effect::Focus { .. }));
    assert!(first_show < first_focus, "shown before focused");
}

#[test]
fn going_to_the_current_workspace_does_nothing() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0)]);

    // Act
    let effects = h.action(Action::GoToWorkspace(1)).to_vec();

    // Assert
    assert!(effects.is_empty());
}

#[test]
fn per_monitor_mode_switches_only_the_focused_monitor() {
    // Arrange
    let mut h = Harness::new(2, &[(1, 0), (2, 1)]);

    // Act
    h.action(Action::GoToWorkspace(2));

    // Assert
    assert_eq!(h.tiler.monitors()[0].active_workspace(), 1);
    assert_eq!(h.tiler.monitors()[1].active_workspace(), 0);
    assert!(!h.desktop.windows[&2].cloaked);
}

#[test]
fn global_mode_switches_every_monitor() {
    // Arrange
    let settings = with_mode(WorkspaceMode::Global, HidingBehaviour::Cloak);
    let mut h = Harness::with_settings(settings, 2, &[(1, 0), (2, 1)]);

    // Act
    h.action(Action::GoToWorkspace(2));

    // Assert
    assert_eq!(h.tiler.monitors()[0].active_workspace(), 1);
    assert_eq!(h.tiler.monitors()[1].active_workspace(), 1);
    assert!(h.desktop.windows[&1].cloaked);
    assert!(h.desktop.windows[&2].cloaked);
}

#[test]
fn sending_a_window_follows_it_to_the_workspace() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);

    // Act
    h.action(Action::SendToWorkspace(3));

    // Assert
    assert_eq!(h.tiler.monitors()[0].active_workspace(), 2);
    assert_eq!(h.active(0), vec![1]);
    assert!(h.workspace(0, 0).contains(2));
    assert!(h.desktop.windows[&2].cloaked);
    assert_eq!(h.tiler.focused_window(), Some(1));
}

#[test]
fn global_mode_send_keeps_the_monitors_in_step() {
    // Arrange
    let settings = with_mode(WorkspaceMode::Global, HidingBehaviour::Cloak);
    let mut h = Harness::with_settings(settings, 2, &[(1, 0), (2, 1)]);

    // Act
    h.action(Action::SendToWorkspace(4));

    // Assert
    assert_eq!(h.tiler.monitors()[0].active_workspace(), 3);
    assert_eq!(h.tiler.monitors()[1].active_workspace(), 3);
    assert_eq!(h.tiler.focused_monitor(), 0);
}

#[test]
fn hidden_windows_shown_in_the_same_step_are_not_pruned() {
    // Arrange -- with hiding that makes windows invisible, the window is invisible
    // until the show is applied, after the step.
    let settings = with_mode(WorkspaceMode::PerMonitor, HidingBehaviour::Hide);
    let mut h = Harness::with_settings(settings, 1, &[(1, 0), (2, 0)]);
    h.action(Action::GoToWorkspace(2));
    assert!(!h.desktop.windows[&1].visible);

    // Act
    h.action(Action::GoToWorkspace(1));

    // Assert
    assert_eq!(h.active(0), vec![1, 2]);
    assert!(h.desktop.windows[&1].visible);
}

#[test]
fn minimize_hiding_keeps_borders_on_windows_being_restored() {
    // Arrange
    let settings = with_mode(WorkspaceMode::PerMonitor, HidingBehaviour::Minimize);
    let mut h = Harness::with_settings(settings, 1, &[(1, 0)]);
    h.action(Action::GoToWorkspace(2));

    // Act
    h.action(Action::GoToWorkspace(1));

    // Assert
    let last_border = h.effects.iter().rev().find_map(|e| match e {
        Effect::Border { window: 1, style } => Some(style.is_some()),
        _ => None,
    });
    assert_eq!(last_border, Some(true));
}

#[test]
fn focusing_a_window_on_a_hidden_workspace_switches_to_it() {
    // Arrange -- window 1 sits on workspace 2, the user is on 1.
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    h.action(Action::SendToWorkspace(2));
    h.action(Action::GoToWorkspace(1));
    assert_eq!(h.active(0), vec![2]);

    // Act -- the user picks window 1 from the taskbar.
    h.event(WindowEvent::Focused { hwnd: 1 });

    // Assert
    assert_eq!(h.tiler.monitors()[0].active_workspace(), 1);
    assert_eq!(h.tiler.focused_window(), Some(1));
    assert!(!h.desktop.windows[&1].cloaked);
}
