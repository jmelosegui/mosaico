//! Key combinations: a key and the modifiers held with it.

use std::fmt;
use std::ops::BitOr;

use crate::config::Modifier;

use super::Key;

/// A set of modifier keys, held down together with a [`Key`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Modifiers(u8);

impl Modifiers {
    /// No modifier at all.
    pub const NONE: Self = Self(0);
    /// The Ctrl key.
    pub const CTRL: Self = Self(1);
    /// The Alt key.
    pub const ALT: Self = Self(1 << 1);
    /// The Shift key.
    pub const SHIFT: Self = Self(1 << 2);
    /// The Windows key, called Super or Command on other systems.
    pub const WIN: Self = Self(1 << 3);

    /// Whether every modifier in `other` is in this set too.
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Whether the set holds no modifier.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

/// The modifiers in either set.
impl BitOr for Modifiers {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl From<Modifier> for Modifiers {
    fn from(modifier: Modifier) -> Self {
        match modifier {
            Modifier::Ctrl => Self::CTRL,
            Modifier::Alt => Self::ALT,
            Modifier::Shift => Self::SHIFT,
            Modifier::Win => Self::WIN,
        }
    }
}

/// Collects the `modifiers` list of a binding. A modifier listed twice
/// counts once.
impl FromIterator<Modifier> for Modifiers {
    fn from_iter<I: IntoIterator<Item = Modifier>>(modifiers: I) -> Self {
        modifiers
            .into_iter()
            .fold(Self::NONE, |set, modifier| set | Self::from(modifier))
    }
}

/// Joined with `+` in the usual order, `Ctrl+Alt+Shift+Win`.
impl fmt::Display for Modifiers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names = [
            (Self::CTRL, "Ctrl"),
            (Self::ALT, "Alt"),
            (Self::SHIFT, "Shift"),
            (Self::WIN, "Win"),
        ];
        let held = names
            .iter()
            .filter(|(modifier, _)| self.contains(*modifier));
        for (i, (_, name)) in held.enumerate() {
            if i > 0 {
                f.write_str("+")?;
            }
            f.write_str(name)?;
        }
        Ok(())
    }
}

/// A key pressed while a set of modifiers is held down.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyCombo {
    /// The modifiers held down.
    pub modifiers: Modifiers,
    /// The key pressed.
    pub key: Key,
}

/// Shown the way shortcuts are written, as `Alt+Shift+J`, or as the key
/// alone when no modifier is held.
impl fmt::Display for KeyCombo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.modifiers.is_empty() {
            write!(f, "{}", self.key)
        } else {
            write!(f, "{}+{}", self.modifiers, self.key)
        }
    }
}

#[cfg(test)]
#[path = "tests/combo_tests.rs"]
mod tests;
