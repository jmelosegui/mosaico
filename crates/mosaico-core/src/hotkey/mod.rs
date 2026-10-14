//! Hotkeys: the keys a keybinding names, and how they are claimed.
//!
//! A binding's key name and modifiers are parsed here, into a
//! [`Key`](crate::hotkey::Key) and a set of
//! [`Modifiers`](crate::hotkey::Modifiers), so the daemon and
//! `mosaico doctor` accept the same names and report the same errors on
//! every platform. [`Hotkeys`](crate::hotkey::Hotkeys) decides which
//! bindings are registered when, including what stays registered while
//! paused, and a platform's
//! [`HotkeyRegistrar`](crate::hotkey::HotkeyRegistrar) carries that out.

mod combo;
mod hotkeys;
mod key;
mod key_names;
mod registrar;
mod rejected;

pub use combo::{KeyCombo, Modifiers};
pub use hotkeys::Hotkeys;
pub use key::{Digit, FunctionKey, Key, Letter};
#[cfg(test)]
pub(crate) use key::{digit, function, letter};
pub use registrar::{HotkeyId, HotkeyRegistrar};
pub use rejected::Rejected;
