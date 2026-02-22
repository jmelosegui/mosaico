use serde::{Deserialize, Serialize};

use crate::Action;
use crate::action::Direction;

/// A user-configured keybinding that maps a key combination to an action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybinding {
    /// The action to trigger.
    pub action: Action,
    /// Key name (e.g. "J", "Enter", "Space", "F1").
    pub key: String,
    /// Modifier keys (e.g. ["alt", "shift"]).
    pub modifiers: Vec<Modifier>,
}

/// Keyboard modifier keys.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Modifier {
    Alt,
    Shift,
    Ctrl,
    Win,
}

/// Returns the default keybindings using vim-style motions.
///
/// Focus: Alt + H/J/K/L (left/down/up/right)
/// Move/Swap: Alt + Shift + H/J/K/L
/// Workspaces: Alt + 1..8 (switch), Alt + Shift + 1..8 (send)
/// Monocle: Alt + T
/// Retile: Alt + Shift + R
pub fn defaults() -> Vec<Keybinding> {
    use Modifier::{Alt, Shift};

    let mut bindings = vec![
        // Focus: spatial navigation
        bind(Action::Focus(Direction::Down), "J", &[Alt]),
        bind(Action::Focus(Direction::Up), "K", &[Alt]),
        bind(Action::Focus(Direction::Right), "L", &[Alt]),
        bind(Action::Focus(Direction::Left), "H", &[Alt]),
        // Move: spatial swap / cross-monitor
        bind(Action::Move(Direction::Down), "J", &[Alt, Shift]),
        bind(Action::Move(Direction::Up), "K", &[Alt, Shift]),
        bind(Action::Move(Direction::Right), "L", &[Alt, Shift]),
        bind(Action::Move(Direction::Left), "H", &[Alt, Shift]),
        // Layout
        bind(Action::Retile, "R", &[Alt, Shift]),
        bind(Action::ToggleMonocle, "T", &[Alt]),
        // Close window
        bind(Action::CloseFocused, "Q", &[Alt]),
    ];

    // Workspaces: Alt+1..8 to switch, Alt+Shift+1..8 to send
    for n in 1..=8u8 {
        let key = n.to_string();
        bindings.push(bind(Action::GoToWorkspace(n), &key, &[Alt]));
        bindings.push(bind(Action::SendToWorkspace(n), &key, &[Alt, Shift]));
    }

    bindings
}

fn bind(action: Action, key: &str, modifiers: &[Modifier]) -> Keybinding {
    Keybinding {
        action,
        key: key.into(),
        modifiers: modifiers.to_vec(),
    }
}
