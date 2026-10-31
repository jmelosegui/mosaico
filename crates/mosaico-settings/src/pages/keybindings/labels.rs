//! What each action is called, and which group of the page it is in.

use mosaico_core::Action;

/// The groups of the page, in the order it shows them.
pub const GROUPS: [&str; 7] = [
    "Focus",
    "Move windows",
    "Workspaces",
    "Layout",
    "Floating windows",
    "Windows",
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
        Action::Retile | Action::ToggleMonocle | Action::CycleLayout => GROUPS[3],
        Action::ToggleFloat
        | Action::CycleFloat
        | Action::GrowWidth
        | Action::ShrinkWidth
        | Action::GrowHeight
        | Action::ShrinkHeight => GROUPS[4],
        Action::CloseFocused | Action::MinimizeFocused => GROUPS[5],
        Action::TogglePause => GROUPS[6],
    }
}

/// What `action` does, as a row names it.
pub fn label(action: Action) -> String {
    match action {
        Action::Focus(dir) => format!("Focus {dir}"),
        Action::Move(dir) => format!("Move window {dir}"),
        Action::GoToWorkspace(n) => format!("Go to workspace {n}"),
        Action::GoToWorkspaceNext => "Go to the next workspace".into(),
        Action::GoToWorkspacePrev => "Go to the previous workspace".into(),
        Action::SendToWorkspace(n) => format!("Send window to workspace {n}"),
        Action::Retile => "Retile windows".into(),
        Action::ToggleMonocle => "Toggle monocle".into(),
        Action::CycleLayout => "Cycle the layout".into(),
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
