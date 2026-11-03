//! What a hotkey does when it is pressed.

use crate::Action;

/// What a press of a registered hotkey asks the daemon to do.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Target {
    /// Run the action of a `[[keybinding]]`.
    Action(Action),
    /// Start the program of a `[[launch]]` binding. The number is the
    /// binding's position in the launch bindings the hotkeys were
    /// registered with.
    Launch(usize),
}
