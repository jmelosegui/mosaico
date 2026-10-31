//! Reading the key combination a user presses while a binding is being
//! changed.

use eframe::egui::{Event, Key, Modifiers};
use mosaico_core::config::Modifier;

/// What a key press means to a binding being changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pressed {
    /// Escape on its own: leave the binding as it is.
    Cancel,
    /// A combination to try. Core checks the key name.
    Combo {
        key: String,
        modifiers: Vec<Modifier>,
    },
    /// A press that cannot be a hotkey, and why.
    Refused(&'static str),
}

/// What the first key press of `events` means, if any. The key is the
/// physical one, so Shift and a digit stay that digit.
pub fn pressed(events: &[Event]) -> Option<Pressed> {
    events.iter().find_map(|event| match event {
        Event::Key {
            key,
            physical_key,
            pressed: true,
            repeat: false,
            modifiers,
            ..
        } => Some(classify(physical_key.unwrap_or(*key), *modifiers)),
        _ => None,
    })
}

fn classify(key: Key, held: Modifiers) -> Pressed {
    let modifiers = modifiers_of(held);
    if key == Key::Escape && modifiers.is_empty() {
        return Pressed::Cancel;
    }
    if modifiers.is_empty() && !is_function(key) {
        return Pressed::Refused("Hold Ctrl, Alt or Shift while you press the key.");
    }
    Pressed::Combo {
        key: key_name(key).into(),
        modifiers,
    }
}

fn is_function(key: Key) -> bool {
    let name = key.name();
    name.strip_prefix('F')
        .is_some_and(|n| n.parse::<u8>().is_ok())
}

fn modifiers_of(held: Modifiers) -> Vec<Modifier> {
    let all = [
        (held.alt, Modifier::Alt),
        (held.shift, Modifier::Shift),
        (held.ctrl, Modifier::Ctrl),
    ];
    all.into_iter()
        .filter(|(on, _)| *on)
        .map(|(_, m)| m)
        .collect()
}

/// The name `keybindings.toml` gives `key`, where egui spells it
/// otherwise. Names core does not know are refused by core.
fn key_name(key: Key) -> &'static str {
    match key {
        Key::OpenBracket => "LBracket",
        Key::CloseBracket => "RBracket",
        other => other.name(),
    }
}

#[cfg(test)]
#[path = "capture_tests.rs"]
mod tests;
