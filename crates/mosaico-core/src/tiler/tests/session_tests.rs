//! A daemon killed with windows hidden, and the next one starting (#67).

use crate::config::{HidingBehaviour, WorkspaceMode};

use super::*;

fn hiding(how: HidingBehaviour) -> Settings {
    let mut settings = Settings::default();
    settings.layout.hiding = how;
    settings.workspaces.mode = WorkspaceMode::PerMonitor;
    settings
}

#[test]
fn windows_hidden_by_a_killed_daemon_are_shown_and_tiled_by_the_next_one() {
    // #67
    // Arrange -- workspace 2 is on screen, so the windows of workspace 1
    // are cloaked when the daemon is killed.
    let mut h = Harness::with_settings(hiding(HidingBehaviour::Cloak), 1, &[(1, 0), (2, 0)]);
    h.action(Action::SendToWorkspace(2));
    h.action(Action::GoToWorkspace(3));
    assert!(h.desktop.windows[&wid(1)].cloaked && h.desktop.windows[&wid(2)].cloaked);

    // Act
    h.kill_and_restart();

    // Assert -- nothing is left invisible, and the new daemon manages both.
    assert!(
        !h.desktop.windows[&wid(1)].cloaked,
        "window 1 is still cloaked after the restart"
    );
    assert!(
        !h.desktop.windows[&wid(2)].cloaked,
        "window 2 is still cloaked after the restart"
    );
    let mut managed = h.active(0);
    managed.sort();
    assert_eq!(managed, vec![1, 2]);
}

#[test]
fn windows_hidden_with_show_window_are_shown_by_the_next_daemon() {
    // #67
    // Arrange
    let mut h = Harness::with_settings(hiding(HidingBehaviour::Hide), 1, &[(1, 0)]);
    h.action(Action::GoToWorkspace(2));
    assert!(!h.desktop.windows[&wid(1)].visible);

    // Act
    h.kill_and_restart();

    // Assert
    assert!(
        h.desktop.windows[&wid(1)].visible,
        "window 1 is still hidden after the restart"
    );
    assert_eq!(h.active(0), vec![1]);
}

#[test]
fn a_clean_shutdown_leaves_nothing_to_restore() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0)]);
    h.action(Action::GoToWorkspace(2));

    // Act
    h.step(Input::Shutdown);

    // Assert
    assert!(h.hidden.is_empty());
}

#[test]
fn a_reused_handle_is_left_alone() {
    // Arrange -- the hidden window closed and another process got its
    // handle, cloaked by something else.
    let mut h = Harness::new(1, &[(1, 0)]);
    h.action(Action::GoToWorkspace(2));
    if let Some(w) = h.desktop.windows.get_mut(&wid(1)) {
        w.process = 4242;
    }

    // Act
    let effects = h.kill_and_restart().to_vec();

    // Assert
    assert!(
        !effects
            .iter()
            .any(|e| matches!(e, Effect::Show { window, .. } if *window == wid(1))),
        "a window that is not the one hidden must not be shown"
    );
    assert!(h.desktop.windows[&wid(1)].cloaked);
}
