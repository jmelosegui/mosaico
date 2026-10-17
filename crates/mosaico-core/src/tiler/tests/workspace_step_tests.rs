//! Stepping through the workspaces with `goto-workspace-next` and
//! `goto-workspace-prev`.

use crate::Action;
use crate::action::Origin;
use crate::config::WorkspaceMode;

use super::super::tests::Harness;
use super::support::{on_screen, stepping};

#[test]
fn next_goes_to_the_workspace_after_the_one_on_screen() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0)]);
    h.action(Action::GoToWorkspace(3));

    // Act
    h.action(Action::GoToWorkspaceNext);

    // Assert
    assert_eq!(on_screen(&h), [4]);
}

#[test]
fn prev_goes_to_the_workspace_before_the_one_on_screen() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0)]);
    h.action(Action::GoToWorkspace(3));

    // Act
    h.action(Action::GoToWorkspacePrev);

    // Assert
    assert_eq!(on_screen(&h), [2]);
}

#[test]
fn without_wrap_prev_on_the_first_workspace_does_nothing() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0)]);

    // Act
    let effects = h.action(Action::GoToWorkspacePrev).to_vec();

    // Assert
    assert!(effects.is_empty());
    assert_eq!(on_screen(&h), [1]);
}

#[test]
fn without_wrap_next_on_the_last_workspace_does_nothing() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0)]);
    h.action(Action::GoToWorkspace(8));

    // Act
    let effects = h.action(Action::GoToWorkspaceNext).to_vec();

    // Assert
    assert!(effects.is_empty());
    assert_eq!(on_screen(&h), [8]);
}

#[test]
fn next_steps_only_the_focused_monitor() {
    // Arrange
    let mut h = Harness::new(2, &[(1, 0), (2, 1)]);

    // Act
    h.action(Action::GoToWorkspaceNext);

    // Assert
    assert_eq!(on_screen(&h), [2, 1]);
}

#[test]
fn in_global_mode_next_steps_every_monitor() {
    // Arrange
    let settings = stepping(WorkspaceMode::Global, false);
    let mut h = Harness::with_settings(settings, 2, &[(1, 0), (2, 1)]);

    // Act
    h.action(Action::GoToWorkspaceNext);

    // Assert
    assert_eq!(on_screen(&h), [2, 2]);
}

#[test]
fn next_from_a_bar_steps_that_bars_monitor() {
    // Arrange
    let mut h = Harness::new(2, &[(1, 0), (2, 1)]);

    // Act
    h.action_from(Action::GoToWorkspaceNext, Origin::Bar(1));

    // Assert
    assert_eq!(on_screen(&h), [1, 2]);
    assert_eq!(h.tiler.focused_monitor(), 1);
}
