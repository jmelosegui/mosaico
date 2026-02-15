use serde::{Deserialize, Serialize};

use crate::Action;

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
/// Monocle: Alt + T
/// Retile: Alt + Shift + R
pub fn defaults() -> Vec<Keybinding> {
    use Modifier::{Alt, Shift};

    vec![
        // Focus within workspace
        bind(Action::FocusNext, "J", &[Alt]),
        bind(Action::FocusPrev, "K", &[Alt]),
        // Focus across monitors
        bind(Action::FocusMonitorNext, "L", &[Alt]),
        bind(Action::FocusMonitorPrev, "H", &[Alt]),
        // Swap within workspace
        bind(Action::SwapNext, "J", &[Alt, Shift]),
        bind(Action::SwapPrev, "K", &[Alt, Shift]),
        // Move across monitors
        bind(Action::MoveToMonitorNext, "L", &[Alt, Shift]),
        bind(Action::MoveToMonitorPrev, "H", &[Alt, Shift]),
        // Layout
        bind(Action::Retile, "R", &[Alt, Shift]),
        bind(Action::ToggleMonocle, "T", &[Alt]),
    ]
}

fn bind(action: Action, key: &str, modifiers: &[Modifier]) -> Keybinding {
    Keybinding {
        action,
        key: key.into(),
        modifiers: modifiers.to_vec(),
    }
}
