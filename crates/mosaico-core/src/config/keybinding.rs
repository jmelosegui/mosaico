use serde::{Deserialize, Serialize};

use crate::action::Direction;
use crate::hotkey::KeyCombo;
use crate::{Action, Result};

/// A user-configured keybinding that maps a key combination to an action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybinding {
    /// The action to trigger.
    pub action: Action,
    /// Key name as written in the file (e.g. "J", "Enter", "Space", "F1").
    ///
    /// Kept as text and checked by [`Keybinding::combo`], so an unknown
    /// name puts only its own binding out of action, not the whole file.
    pub key: String,
    /// Modifier keys (e.g. ["alt", "shift"]).
    pub modifiers: Vec<Modifier>,
}

impl Keybinding {
    /// The key combination that triggers this binding.
    ///
    /// This is where a binding's key name is checked, for the daemon and
    /// for `mosaico doctor` alike, so both accept the same names on every
    /// platform.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Config`](crate::Error::Config) when the key name
    /// is not one mosaico knows.
    pub fn combo(&self) -> Result<KeyCombo> {
        Ok(KeyCombo {
            modifiers: self.modifiers.iter().copied().collect(),
            key: self.key.parse()?,
        })
    }
}

/// Keyboard modifier keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Modifier {
    /// Alt key.
    Alt,
    /// Shift key.
    Shift,
    /// Control key.
    Ctrl,
    /// Windows/Super key.
    Win,
}

/// Returns the default keybindings using vim-style motions.
///
/// Focus: Alt + H/J/K/L (left/down/up/right)
/// Move/Swap: Alt + Shift + H/J/K/L
/// Workspaces: Alt + 1..8 (switch), Alt + Shift + 1..8 (send)
/// Monocle: Alt + T
/// Floating: Alt + F (toggle), Alt + Shift + F (cycle),
/// Alt + Period/Comma (width), Alt + Shift + Period/Comma (height)
/// Retile: Alt + Shift + R
/// Pause hotkeys: Alt + Shift + P
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
        // Floating windows
        bind(Action::ToggleFloat, "F", &[Alt]),
        bind(Action::CycleFloat, "F", &[Alt, Shift]),
        bind(Action::GrowWidth, "Period", &[Alt]),
        bind(Action::ShrinkWidth, "Comma", &[Alt]),
        bind(Action::GrowHeight, "Period", &[Alt, Shift]),
        bind(Action::ShrinkHeight, "Comma", &[Alt, Shift]),
        // Close window
        bind(Action::CloseFocused, "Q", &[Alt]),
        // Minimize window
        bind(Action::MinimizeFocused, "M", &[Alt]),
        // Cycle layout
        bind(Action::CycleLayout, "N", &[Alt]),
        // Pause/unpause all hotkeys
        bind(Action::TogglePause, "P", &[Alt, Shift]),
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

#[cfg(test)]
#[path = "keybinding_tests.rs"]
mod tests;
