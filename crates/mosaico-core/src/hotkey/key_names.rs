//! The names keys are written with in `keybindings.toml`.

use std::fmt;
use std::str::FromStr;

use crate::Error;

use super::{Digit, FunctionKey, Key, Letter};

impl FromStr for Key {
    type Err = Error;

    /// Parses a key name, ignoring ASCII case: a letter, a digit, `F` and
    /// the number of a function key, or the name of one of the other keys.
    fn from_str(name: &str) -> Result<Self, Self::Err> {
        let upper = name.to_ascii_uppercase();
        if let [byte] = upper.as_bytes() {
            if let Some(letter) = Letter::new(char::from(*byte)) {
                return Ok(Self::Letter(letter));
            }
            if let Some(digit) = byte.checked_sub(b'0').and_then(Digit::new) {
                return Ok(Self::Digit(digit));
            }
        }
        let number = upper.strip_prefix('F').and_then(|n| n.parse::<u32>().ok());
        let function = number.and_then(|n| u8::try_from(n).ok());
        if let Some(function) = function.and_then(FunctionKey::new) {
            return Ok(Self::Function(function));
        }
        named(&upper).ok_or_else(|| unknown(name))
    }
}

/// The key that an uppercased name other than a letter, a digit or a
/// function key stands for.
fn named(upper: &str) -> Option<Key> {
    let key = match upper {
        "ENTER" | "RETURN" => Key::Enter,
        "TAB" => Key::Tab,
        "ESCAPE" | "ESC" => Key::Escape,
        "SPACE" => Key::Space,
        "BACKSPACE" => Key::Backspace,
        "DELETE" | "DEL" => Key::Delete,
        "INSERT" | "INS" => Key::Insert,
        "HOME" => Key::Home,
        "END" => Key::End,
        "PAGEUP" | "PGUP" => Key::PageUp,
        "PAGEDOWN" | "PGDN" => Key::PageDown,
        "LEFT" => Key::Left,
        "UP" => Key::Up,
        "RIGHT" => Key::Right,
        "DOWN" => Key::Down,
        "MINUS" => Key::Minus,
        "PLUS" | "EQUALS" => Key::Plus,
        "COMMA" => Key::Comma,
        "PERIOD" | "DOT" => Key::Period,
        "SLASH" => Key::Slash,
        "SEMICOLON" => Key::Semicolon,
        "BACKSLASH" => Key::Backslash,
        "LBRACKET" => Key::LBracket,
        "RBRACKET" => Key::RBracket,
        "QUOTE" => Key::Quote,
        "BACKTICK" | "GRAVE" => Key::Backtick,
        _ => return None,
    };
    Some(key)
}

/// The error for a name that is not a key.
fn unknown(name: &str) -> Error {
    let message = format!(
        "unknown key name {name:?} (expected A-Z, 0-9, F1-F{} or a name \
         such as Enter, Space or PageUp)",
        FunctionKey::LAST
    );
    Error::config(None, message)
}

/// Letters and digits as themselves, function keys as `F1`, and the other
/// keys by their first name, such as `Escape` rather than `Esc`.
impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Letter(letter) => return write!(f, "{}", letter.get()),
            Self::Digit(digit) => return write!(f, "{}", digit.get()),
            Self::Function(function) => return write!(f, "F{}", function.get()),
            Self::Enter => "Enter",
            Self::Tab => "Tab",
            Self::Escape => "Escape",
            Self::Space => "Space",
            Self::Backspace => "Backspace",
            Self::Delete => "Delete",
            Self::Insert => "Insert",
            Self::Home => "Home",
            Self::End => "End",
            Self::PageUp => "PageUp",
            Self::PageDown => "PageDown",
            Self::Left => "Left",
            Self::Up => "Up",
            Self::Right => "Right",
            Self::Down => "Down",
            Self::Minus => "Minus",
            Self::Plus => "Plus",
            Self::Comma => "Comma",
            Self::Period => "Period",
            Self::Slash => "Slash",
            Self::Semicolon => "Semicolon",
            Self::Backslash => "Backslash",
            Self::LBracket => "LBracket",
            Self::RBracket => "RBracket",
            Self::Quote => "Quote",
            Self::Backtick => "Backtick",
        };
        f.write_str(name)
    }
}

#[cfg(test)]
#[path = "tests/key_tests.rs"]
mod tests;
