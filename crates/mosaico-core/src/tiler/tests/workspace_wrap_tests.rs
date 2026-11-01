//! Stepping round from the last workspace to the first with
//! `[workspaces] wrap`.

use crate::Action;
use crate::config::{Config, WorkspaceMode};

use super::super::Input;
use super::super::tests::Harness;
use super::support::{on_screen, stepping};

#[test]
fn with_wrap_next_on_the_last_workspace_goes_to_the_first() {
    // Arrange
    let mut h = Harness::with_settings(stepping(WorkspaceMode::PerMonitor, true), 1, &[(1, 0)]);
    h.action(Action::GoToWorkspace(8));

    // Act
    h.action(Action::GoToWorkspaceNext);

    // Assert
    assert_eq!(on_screen(&h), [1]);
}

#[test]
fn with_wrap_prev_on_the_first_workspace_goes_to_the_last() {
    // Arrange
    let mut h = Harness::with_settings(stepping(WorkspaceMode::PerMonitor, true), 1, &[(1, 0)]);

    // Act
    h.action(Action::GoToWorkspacePrev);

    // Assert
    assert_eq!(on_screen(&h), [8]);
}

#[test]
fn a_reload_takes_the_new_wrap() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0)]);
    let mut config = Config::with_gap(8);
    config.workspaces.wrap = true;
    h.step(Input::Config(Box::new(config)));

    // Act
    h.action(Action::GoToWorkspacePrev);

    // Assert
    assert_eq!(on_screen(&h), [8]);
}
