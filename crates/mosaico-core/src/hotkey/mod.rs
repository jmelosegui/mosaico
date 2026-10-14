//! The keys a keybinding names, as every platform reads them.
//!
//! A binding's key name and modifiers are parsed here, into a
//! [`Key`](crate::hotkey::Key) and a set of
//! [`Modifiers`](crate::hotkey::Modifiers), so the daemon and
//! `mosaico doctor` accept the same names and report the same errors on
//! every platform.

mod combo;
mod key;
mod key_names;

pub use combo::{KeyCombo, Modifiers};
pub use key::{Digit, FunctionKey, Key, Letter};
#[cfg(test)]
pub(crate) use key::{digit, function, letter};
