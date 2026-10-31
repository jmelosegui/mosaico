use crate::{Action, Direction};

use super::*;

#[test]
fn the_actions_about_the_focused_window_act_on_it() {
    // Arrange
    let actions = [
        Action::Focus(Direction::Left),
        Action::Move(Direction::Down),
        Action::SendToWorkspace(2),
        Action::CloseFocused,
        Action::MinimizeFocused,
        Action::ToggleFloat,
        Action::GrowWidth,
        Action::ShrinkHeight,
    ];

    // Act / Assert
    for action in actions {
        assert_eq!(action.scope(), Scope::FocusedWindow, "{action}");
    }
}

#[test]
fn the_layout_monocle_and_workspace_switches_act_on_a_monitor() {
    // Arrange
    let actions = [
        Action::CycleLayout,
        Action::ToggleMonocle,
        Action::GoToWorkspace(3),
        Action::GoToWorkspaceNext,
        Action::GoToWorkspacePrev,
        Action::CycleFloat,
    ];

    // Act / Assert
    for action in actions {
        assert_eq!(action.scope(), Scope::Monitor, "{action}");
    }
}

#[test]
fn retiling_and_pausing_act_on_the_whole_daemon() {
    // Act / Assert
    assert_eq!(Action::Retile.scope(), Scope::Daemon);
    assert_eq!(Action::TogglePause.scope(), Scope::Daemon);
}
