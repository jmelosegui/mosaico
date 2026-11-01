//! What each action is called, and which group of the page it is in.

use mosaico_core::{Action, Direction};

/// The groups of the page, in the order it shows them.
pub const GROUPS: [&str; 5] = [
    "Focus",
    "Move",
    "Workspaces",
    "Windows and layout",
    "Mosaico",
];

/// The group of `action`, one of [`GROUPS`].
pub fn group(action: Action) -> &'static str {
    match action {
        Action::Focus(_) => GROUPS[0],
        Action::Move(_) => GROUPS[1],
        Action::GoToWorkspace(_)
        | Action::GoToWorkspaceNext
        | Action::GoToWorkspacePrev
        | Action::SendToWorkspace(_) => GROUPS[2],
        Action::Retile
        | Action::ToggleMonocle
        | Action::CycleLayout
        | Action::ToggleFloat
        | Action::CycleFloat
        | Action::GrowWidth
        | Action::ShrinkWidth
        | Action::GrowHeight
        | Action::ShrinkHeight
        | Action::CloseFocused
        | Action::MinimizeFocused => GROUPS[3],
        Action::TogglePause => GROUPS[4],
    }
}

/// Where `action` sits among the others of its group.
pub fn rank(action: Action) -> u8 {
    let direction = |d: Direction| match d {
        Direction::Left => 0,
        Direction::Down => 1,
        Direction::Up => 2,
        Direction::Right => 3,
    };
    match action {
        Action::Focus(d) | Action::Move(d) => direction(d),
        Action::GoToWorkspace(n) => n,
        Action::GoToWorkspaceNext => 20,
        Action::GoToWorkspacePrev => 21,
        Action::SendToWorkspace(n) => 30 + n,
        Action::ToggleMonocle => 0,
        Action::CycleLayout => 1,
        Action::Retile => 2,
        Action::ToggleFloat => 3,
        Action::CycleFloat => 4,
        Action::GrowWidth => 5,
        Action::ShrinkWidth => 6,
        Action::GrowHeight => 7,
        Action::ShrinkHeight => 8,
        Action::MinimizeFocused => 9,
        Action::CloseFocused => 10,
        Action::TogglePause => 0,
    }
}

/// What `action` does, as a row names it.
pub fn label(action: Action) -> String {
    match action {
        Action::Focus(dir) => format!("Focus {dir}"),
        Action::Move(dir) => format!("Move {dir}"),
        Action::GoToWorkspace(n) => format!("Go to workspace {n}"),
        Action::GoToWorkspaceNext => "Go to the next workspace".into(),
        Action::GoToWorkspacePrev => "Go to the previous workspace".into(),
        Action::SendToWorkspace(n) => format!("Send window to workspace {n}"),
        Action::Retile => "Retile".into(),
        Action::ToggleMonocle => "Toggle monocle".into(),
        Action::CycleLayout => "Cycle layout".into(),
        Action::ToggleFloat => "Float or tile the window".into(),
        Action::CycleFloat => "Focus the next floating window".into(),
        Action::GrowWidth => "Widen floating window".into(),
        Action::ShrinkWidth => "Narrow floating window".into(),
        Action::GrowHeight => "Heighten floating window".into(),
        Action::ShrinkHeight => "Shorten floating window".into(),
        Action::CloseFocused => "Close window".into(),
        Action::MinimizeFocused => "Minimize window".into(),
        Action::TogglePause => "Pause hotkeys".into(),
    }
}

#[cfg(test)]
mod tests {
    use mosaico_core::config::keybinding;

    use super::*;

    #[test]
    fn every_default_action_has_a_group_and_a_distinct_label() {
        let actions: Vec<_> = keybinding::defaults().iter().map(|b| b.action).collect();
        let mut labels: Vec<_> = actions.iter().map(|a| label(*a)).collect();
        labels.sort();
        labels.dedup();

        assert_eq!(labels.len(), actions.len());
        assert!(actions.iter().all(|a| GROUPS.contains(&group(*a))));
    }
}
