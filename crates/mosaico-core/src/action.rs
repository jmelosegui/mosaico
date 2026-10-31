use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[path = "action_names.rs"]
mod names;
#[path = "action_origin.rs"]
mod origin;
#[path = "action_step.rs"]
mod step;

pub use origin::{Origin, Scope};
pub use step::workspace_after;

/// Spatial direction for focus and move actions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    /// Left.
    Left,
    /// Right.
    Right,
    /// Up.
    Up,
    /// Down.
    Down,
}

impl FromStr for Direction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "left" => Ok(Direction::Left),
            "right" => Ok(Direction::Right),
            "up" => Ok(Direction::Up),
            "down" => Ok(Direction::Down),
            _ => Err(format!("unknown direction: {s}")),
        }
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Direction::Left => write!(f, "left"),
            Direction::Right => write!(f, "right"),
            Direction::Up => write!(f, "up"),
            Direction::Down => write!(f, "down"),
        }
    }
}

/// An action that can be triggered by a hotkey or CLI command.
///
/// Focus and Move each take a spatial [`Direction`], keeping the
/// direction logic in one place instead of duplicating it across
/// separate Next/Prev variants.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum Action {
    /// Move focus in the given direction.
    ///
    /// Left/Right: spatial horizontal neighbor, overflows to adjacent monitor.
    /// Up/Down: spatial vertical neighbor, stops at boundary.
    Focus(Direction),
    /// Move (swap) the focused window in the given direction.
    ///
    /// Left/Right: spatial horizontal swap, overflows to adjacent monitor.
    /// Up/Down: spatial vertical swap, stops at boundary.
    Move(Direction),
    /// Re-apply the current layout to all managed windows.
    Retile,
    /// Toggle monocle mode (focused window fills the monitor).
    ToggleMonocle,
    /// Close the currently focused window.
    CloseFocused,
    /// Switch to workspace N (1-8) on the focused monitor.
    GoToWorkspace(u8),
    /// Switch to the workspace after the one on screen, on the focused
    /// monitor. Past the last one it goes back to the first with
    /// `[workspaces] wrap`, and stays on the last otherwise.
    GoToWorkspaceNext,
    /// Switch to the workspace before the one on screen, the way
    /// [`GoToWorkspaceNext`](Self::GoToWorkspaceNext) steps.
    GoToWorkspacePrev,
    /// Send the focused window to workspace N (1-8) on the same monitor.
    SendToWorkspace(u8),
    /// Minimize the currently focused window.
    MinimizeFocused,
    /// Cycle to the next layout on the focused workspace.
    CycleLayout,
    /// Toggle hotkey pause: unregister all hotkeys (paused) or re-register them (unpaused).
    TogglePause,
    /// Float the focused tiled window above the tiles, or tile it again.
    ToggleFloat,
    /// Focus the next floating window of the workspace on screen.
    CycleFloat,
    /// Widen the focused floating window by one step, around its center.
    GrowWidth,
    /// Narrow the focused floating window by one step, around its center.
    ShrinkWidth,
    /// Heighten the focused floating window by one step, around its center.
    GrowHeight,
    /// Shorten the focused floating window by one step, around its center.
    ShrinkHeight,
}

/// Maximum number of workspaces per monitor.
pub const MAX_WORKSPACES: u8 = 8;

impl TryFrom<String> for Action {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl From<Action> for String {
    fn from(a: Action) -> String {
        a.to_string()
    }
}

#[cfg(test)]
#[path = "action_tests.rs"]
mod tests;
