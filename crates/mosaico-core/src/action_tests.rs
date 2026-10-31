use super::*;

#[test]
fn roundtrip_all_actions() {
    let actions = [
        Action::Focus(Direction::Left),
        Action::Focus(Direction::Right),
        Action::Focus(Direction::Up),
        Action::Focus(Direction::Down),
        Action::Move(Direction::Left),
        Action::Move(Direction::Right),
        Action::Move(Direction::Up),
        Action::Move(Direction::Down),
        Action::Retile,
        Action::ToggleMonocle,
        Action::CloseFocused,
        Action::MinimizeFocused,
        Action::GoToWorkspace(1),
        Action::GoToWorkspace(8),
        Action::SendToWorkspace(1),
        Action::SendToWorkspace(8),
        Action::CycleLayout,
        Action::TogglePause,
        Action::ToggleFloat,
        Action::CycleFloat,
        Action::GrowWidth,
        Action::ShrinkWidth,
        Action::GrowHeight,
        Action::ShrinkHeight,
    ];
    for action in &actions {
        let s = action.to_string();
        let parsed: Action = s.parse().unwrap();
        assert_eq!(&parsed, action, "roundtrip failed for {s}");
    }
}

#[test]
fn unknown_action_returns_error() {
    let result: Result<Action, _> = "fly-away".parse();
    assert!(result.is_err());
}

#[test]
fn unknown_direction_returns_error() {
    let result: Result<Action, _> = "focus-diagonal".parse();
    assert!(result.is_err());
}

#[test]
fn serde_roundtrip() {
    let action = Action::Focus(Direction::Left);
    let json = serde_json::to_string(&action).unwrap();
    assert_eq!(json, "\"focus-left\"");
    let parsed: Action = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, action);
}

#[test]
fn workspace_out_of_range() {
    assert!("goto-workspace-0".parse::<Action>().is_err());
    assert!("goto-workspace-9".parse::<Action>().is_err());
    assert!("send-to-workspace-0".parse::<Action>().is_err());
    assert!("send-to-workspace-9".parse::<Action>().is_err());
}

#[test]
fn stepping_through_workspaces_is_named_next_and_prev() {
    // Act
    let next: Action = "goto-workspace-next".parse().unwrap();
    let prev: Action = "goto-workspace-prev".parse().unwrap();

    // Assert
    assert_eq!(next, Action::GoToWorkspaceNext);
    assert_eq!(prev, Action::GoToWorkspacePrev);
    assert_eq!(next.to_string(), "goto-workspace-next");
    assert_eq!(prev.to_string(), "goto-workspace-prev");
}

#[test]
fn a_step_name_that_is_not_next_or_prev_is_not_a_workspace() {
    // Act
    let result = "goto-workspace-last".parse::<Action>();

    // Assert
    assert!(result.is_err());
}

#[test]
fn an_origin_on_a_bar_names_its_monitor_and_the_pointer() {
    // Act
    let bar = Origin::Bar(1);

    // Assert
    assert_eq!(bar.monitor(), Some(1));
    assert!(bar.from_pointer());
    assert_eq!(Origin::default(), Origin::Focused);
    assert_eq!(Origin::Focused.monitor(), None);
    assert!(!Origin::Focused.from_pointer());
}
