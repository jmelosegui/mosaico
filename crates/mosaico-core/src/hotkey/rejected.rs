//! Keybindings the system did not go along with, and why.

use std::fmt;

use crate::Error;

use super::KeyCombo;

/// A binding whose hotkey could not be registered, or could not be given
/// back for a pause, and why.
#[derive(Debug)]
pub struct Rejected {
    /// What the binding does, as the user knows it: its action, such as
    /// `focus-down`, or `launch` and its command.
    pub binding: String,
    /// Its key combination, unless the binding does not make one.
    pub combo: Option<KeyCombo>,
    /// What went wrong.
    pub error: Error,
}

impl Rejected {
    pub(super) fn new(binding: String, combo: Option<KeyCombo>, error: Error) -> Self {
        Self {
            binding,
            combo,
            error,
        }
    }
}

/// Shown as `Alt+J (focus-down): why`, or as `focus-down: why` when the
/// binding has no combination, since the reason then quotes its key name.
impl fmt::Display for Rejected {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.combo {
            Some(combo) => write!(f, "{combo} ({}): {}", self.binding, self.error),
            None => write!(f, "{}: {}", self.binding, self.error),
        }
    }
}
