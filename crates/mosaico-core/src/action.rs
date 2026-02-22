use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Spatial direction for focus and move actions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Left,
    Right,
    Up,
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
}

impl FromStr for Action {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(dir) = s.strip_prefix("focus-") {
            return Ok(Action::Focus(dir.parse()?));
        }
        if let Some(dir) = s.strip_prefix("move-") {
            return Ok(Action::Move(dir.parse()?));
        }
        match s {
            "retile" => Ok(Action::Retile),
            "toggle-monocle" => Ok(Action::ToggleMonocle),
            "close-focused" => Ok(Action::CloseFocused),
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
        }
    }
}

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
mod tests {
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
}
