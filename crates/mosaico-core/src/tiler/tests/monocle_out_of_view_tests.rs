//! A display change that moves windows onto a monocle workspace out of
//! view (#132).
//!
//! A window a display change brings there stays hidden, and waits behind
//! the monocle window until the workspace is switched to: it is put behind
//! the monocle window then, and placed at the monocle rect.

use crate::config::{HidingBehaviour, WorkspaceMode};

use super::*;

/// Connects the first `count` monitors of [`displays`].
fn connect(h: &mut Harness, count: usize) {
    h.step(Input::Displays {
        monitors: displays(count),
        bar: BarReservation::default(),
    });
}

/// Window 1 is in monocle on the primary's workspace 2, while the
/// primary shows workspace 1, with window 3. Window 2 is on the second
/// monitor's workspace 2, which is not in view either. Windows out of
/// view are hidden `how`.
fn monocle_out_of_view(how: HidingBehaviour) -> Harness {
    let mut settings = Settings::default();
    settings.workspaces.mode = WorkspaceMode::PerMonitor;
    settings.layout.hiding = how;
    let mut h = Harness::with_settings(settings, 2, &[(1, 0), (3, 0), (2, 1)]);
    // Window 1 has focus at the start.
    h.action(Action::SendToWorkspace(2));
    h.action(Action::ToggleMonocle);
    h.action(Action::GoToWorkspace(1));
    h.event(WindowEvent::Focused { window: wid(2) });
    h.action(Action::SendToWorkspace(2));
    h.action(Action::GoToWorkspace(1));
    h
}

#[test]
fn a_window_moved_onto_a_monocle_workspace_out_of_view_stays_hidden() {
    // Arrange
    let mut h = monocle_out_of_view(HidingBehaviour::Cloak);

    // Act -- window 2 moves to the primary's workspace 2.
    connect(&mut h, 1);

    // Assert
    assert!(h.workspace(0, 1).contains(wid(2)));
    assert!(h.desktop.windows[&wid(2)].cloaked);
    let touched = |e: &Effect| match *e {
        Effect::Place { window, .. } | Effect::Show { window, .. } => window == wid(2),
        _ => false,
    };
    assert!(!h.effects.iter().any(touched));
    assert_eq!(h.active(0), vec![3]);
}

/// Moves window 2 onto the primary's monocle workspace 2, out of view,
/// then switches there, and checks that it waits behind window 1.
fn switch_to_monocle_after_a_move(how: HidingBehaviour) {
    // Arrange
    let mut h = monocle_out_of_view(how);
    connect(&mut h, 1);

    // Act
    h.action(Action::GoToWorkspace(2));

    // Assert -- #132: window 2 used to be shown at the second monitor's
    // place.
    assert_eq!(h.rect_of(2), h.tiler.monocle_rect(0));
    assert!(h.desktop.in_front(wid(1), wid(2)));
    assert!(h.workspace(0, 1).monocle());
    assert_eq!(h.rect_of(1), h.tiler.monocle_rect(0));
    assert_eq!(h.tiler.focused_window(), Some(wid(1)));
}

#[test]
fn switching_to_a_monocle_workspace_a_window_was_moved_onto_puts_it_behind() {
    switch_to_monocle_after_a_move(HidingBehaviour::Cloak);
}

#[test]
fn switching_to_a_monocle_workspace_a_minimized_window_was_moved_onto_puts_it_behind() {
    switch_to_monocle_after_a_move(HidingBehaviour::Minimize);
}

#[test]
fn a_window_with_focus_that_lands_out_of_view_waits_behind_the_monocle_window() {
    // Arrange -- window 1 is in monocle on the primary's workspace 2,
    // while the primary shows workspace 1, with window 3. Window 2 has
    // focus on the second monitor's workspace 2.
    let mut h = Harness::new(2, &[(1, 0), (3, 0), (2, 1)]);
    // Window 1 has focus at the start.
    h.action(Action::SendToWorkspace(2));
    h.action(Action::ToggleMonocle);
    h.action(Action::GoToWorkspace(1));
    h.event(WindowEvent::Focused { window: wid(2) });
    h.action(Action::SendToWorkspace(2));

    // Act -- window 2 moves to the primary's workspace 2, out of view, so
    // focus goes to window 3; then the primary switches there.
    connect(&mut h, 1);
    h.action(Action::GoToWorkspace(2));

    // Assert -- #132: window 2 used to be shown at the second monitor's
    // place. Window 1 is still the monocle window, and has focus.
    assert_eq!(h.rect_of(2), h.tiler.monocle_rect(0));
    assert!(h.desktop.in_front(wid(1), wid(2)));
    assert_eq!(h.workspace(0, 1).monocle_window(), Some(wid(1)));
    assert_eq!(h.tiler.focused_window(), Some(wid(1)));
}
