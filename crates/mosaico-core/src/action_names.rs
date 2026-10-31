//! The names of actions, as keys, rules and the CLI spell them.

use std::fmt;
use std::str::FromStr;

use super::{Action, MAX_WORKSPACES};

impl FromStr for Action {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(dir) = s.strip_prefix("focus-") {
            return Ok(Action::Focus(dir.parse()?));
        }
        if let Some(dir) = s.strip_prefix("move-") {
            return Ok(Action::Move(dir.parse()?));
        }
        if let Some(n) = s.strip_prefix("goto-workspace-") {
            return Ok(match n {
                "next" => Action::GoToWorkspaceNext,
                "prev" => Action::GoToWorkspacePrev,
                n => Action::GoToWorkspace(parse_workspace_num(n)?),
            });
        }
        if let Some(n) = s.strip_prefix("send-to-workspace-") {
            return Ok(Action::SendToWorkspace(parse_workspace_num(n)?));
        }
        match s {
            "retile" => Ok(Action::Retile),
            "toggle-monocle" => Ok(Action::ToggleMonocle),
            "close-focused" => Ok(Action::CloseFocused),
            "minimize-focused" => Ok(Action::MinimizeFocused),
            "cycle-layout" => Ok(Action::CycleLayout),
            "toggle-pause" => Ok(Action::TogglePause),
            "toggle-float" => Ok(Action::ToggleFloat),
            "cycle-float" => Ok(Action::CycleFloat),
            "grow-width" => Ok(Action::GrowWidth),
            "shrink-width" => Ok(Action::ShrinkWidth),
            "grow-height" => Ok(Action::GrowHeight),
            "shrink-height" => Ok(Action::ShrinkHeight),
            _ => Err(format!("unknown action: {s}")),
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Focus(dir) => write!(f, "focus-{dir}"),
            Action::Move(dir) => write!(f, "move-{dir}"),
            Action::Retile => write!(f, "retile"),
            Action::ToggleMonocle => write!(f, "toggle-monocle"),
            Action::CloseFocused => write!(f, "close-focused"),
            Action::MinimizeFocused => write!(f, "minimize-focused"),
            Action::CycleLayout => write!(f, "cycle-layout"),
            Action::TogglePause => write!(f, "toggle-pause"),
            Action::ToggleFloat => write!(f, "toggle-float"),
            Action::CycleFloat => write!(f, "cycle-float"),
            Action::GrowWidth => write!(f, "grow-width"),
            Action::ShrinkWidth => write!(f, "shrink-width"),
            Action::GrowHeight => write!(f, "grow-height"),
            Action::ShrinkHeight => write!(f, "shrink-height"),
            Action::GoToWorkspace(n) => write!(f, "goto-workspace-{n}"),
            Action::GoToWorkspaceNext => write!(f, "goto-workspace-next"),
            Action::GoToWorkspacePrev => write!(f, "goto-workspace-prev"),
            Action::SendToWorkspace(n) => write!(f, "send-to-workspace-{n}"),
        }
    }
}

fn parse_workspace_num(s: &str) -> Result<u8, String> {
    let n: u8 = s
        .parse()
        .map_err(|_| format!("invalid workspace number: {s}"))?;
    if (1..=MAX_WORKSPACES).contains(&n) {
        Ok(n)
    } else {
        Err(format!("workspace must be 1-{MAX_WORKSPACES}, got {n}"))
    }
}
