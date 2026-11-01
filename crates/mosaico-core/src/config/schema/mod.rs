//! What each setting in a config file is: its key, what it is called and
//! what it does, and which values it takes.
//!
//! The settings window draws its rows from these descriptors and checks
//! an edit against them before it is written, so a value is described
//! once, here, and not again by every program that edits it.

mod bar_settings;
mod config_settings;
mod review;

pub use bar_settings::{BAR_SETTINGS, bar_default, find_bar};
pub use config_settings::{CONFIG_SETTINGS, config_default};
pub use review::{SettingProblem, review};

use std::fmt;

/// The value of one setting.
#[derive(Debug, Clone, PartialEq)]
pub enum SettingValue {
    /// On or off.
    Bool(bool),
    /// A whole number.
    Int(i64),
    /// A number with a fraction.
    Float(f64),
    /// Text: a name from a list, a color, a curve.
    Text(String),
}

impl SettingValue {
    /// The value `toml` holds, if it is a single one rather than a table
    /// or a list.
    pub fn from_toml(value: &toml::Value) -> Option<Self> {
        match value {
            toml::Value::Boolean(b) => Some(Self::Bool(*b)),
            toml::Value::Integer(n) => Some(Self::Int(*n)),
            toml::Value::Float(x) => Some(Self::Float(*x)),
            toml::Value::String(s) => Some(Self::Text(s.clone())),
            _ => None,
        }
    }

    /// The text, if this is text.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(s) => Some(s),
            _ => None,
        }
    }
}

impl fmt::Display for SettingValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool(true) => f.write_str("on"),
            Self::Bool(false) => f.write_str("off"),
            Self::Int(n) => write!(f, "{n}"),
            Self::Float(x) => write!(f, "{x}"),
            Self::Text(s) => f.write_str(s),
        }
    }
}

/// Which values a setting takes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SettingKind {
    /// On or off.
    Bool,
    /// A whole number, counted in a unit.
    Int {
        /// The least it may be.
        min: i64,
        /// The most it may be.
        max: i64,
        /// What it counts, such as `px`; empty when it has no unit.
        unit: &'static str,
    },
    /// A number with a fraction, edited in steps.
    Float {
        /// The least it may be.
        min: f64,
        /// The most it may be.
        max: f64,
        /// How much one step of the control changes it.
        step: f64,
    },
    /// One of the values listed, each with how it is shown.
    Choice(&'static [(&'static str, &'static str)]),
    /// A color: a `#rrggbb` hex value or the name of a theme color. Empty
    /// means the theme's own.
    Color,
    /// Text the file's loader checks, such as a theme name or a curve.
    Text,
}

/// One setting of a config file.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SettingDescriptor {
    /// Where it is in its file: the tables it is in and its name, joined
    /// with dots, such as `layout.gap`.
    pub key: &'static str,
    /// What it is called.
    pub label: &'static str,
    /// What it does, in a sentence.
    pub help: &'static str,
    /// Which values it takes.
    pub kind: SettingKind,
}

impl SettingDescriptor {
    /// The tables the setting is in, and its name in the last of them.
    pub fn path(&self) -> (Vec<&'static str>, &'static str) {
        let mut parts: Vec<_> = self.key.split('.').collect();
        let name = parts.pop().unwrap_or(self.key);
        (parts, name)
    }

    /// Whether the setting is a color that can also be `"none"`, which
    /// draws nothing.
    pub fn allows_none(&self) -> bool {
        self.kind == SettingKind::Color && self.key == "borders.colors.unfocused"
    }

    /// Whether `value` is one this setting takes.
    ///
    /// # Errors
    ///
    /// Says why not: the value is of another kind, out of range, or not
    /// one of the names listed.
    pub fn check(&self, value: &SettingValue) -> Result<(), String> {
        use SettingValue as V;
        let key = self.key;
        match (self.kind, value) {
            (SettingKind::Bool, V::Bool(_)) => Ok(()),
            (SettingKind::Int { min, max, .. }, V::Int(n)) if (min..=max).contains(n) => Ok(()),
            (SettingKind::Int { min, max, .. }, V::Int(n)) => {
                Err(format!("{key} must be from {min} to {max}, not {n}"))
            }
            (SettingKind::Float { min, max, .. }, V::Float(x)) if (min..=max).contains(x) => Ok(()),
            (SettingKind::Float { min, max, .. }, V::Float(x)) => {
                Err(format!("{key} must be from {min} to {max}, not {x}"))
            }
            (SettingKind::Choice(names), V::Text(s)) if names.iter().any(|(n, _)| n == s) => Ok(()),
            (SettingKind::Choice(names), V::Text(s)) => {
                let names: Vec<_> = names.iter().map(|(n, _)| *n).collect();
                Err(format!(
                    "{key} must be one of {}, not {s:?}",
                    names.join(", ")
                ))
            }
            (SettingKind::Color | SettingKind::Text, V::Text(_)) => Ok(()),
            (_, other) => Err(format!("{key} does not take {other}")),
        }
    }

    /// `value`, read from a file, as this setting takes it: a whole
    /// number where a fraction is expected becomes one, and a number out
    /// of range is brought into it, as loading the file does. A value of
    /// another kind, or a name not listed, gives `None`.
    pub fn normalize(&self, value: SettingValue) -> Option<SettingValue> {
        use SettingValue as V;
        let value = match (self.kind, value) {
            (SettingKind::Int { min, max, .. }, V::Int(n)) => V::Int(n.clamp(min, max)),
            (SettingKind::Float { min, max, .. }, V::Int(n)) => {
                V::Float((n as f64).clamp(min, max))
            }
            (SettingKind::Float { min, max, .. }, V::Float(x)) => V::Float(x.clamp(min, max)),
            (_, other) => other,
        };
        self.check(&value).is_ok().then_some(value)
    }
}

/// The setting of `config.toml` at `key`.
pub fn find(key: &str) -> Option<&'static SettingDescriptor> {
    CONFIG_SETTINGS.iter().find(|s| s.key == key)
}

/// The single value `defaults`, a config with its defaults, has at `key`.
fn default_at(defaults: &impl serde::Serialize, key: &str) -> Option<SettingValue> {
    let mut value = toml::Value::try_from(defaults).ok()?;
    for part in key.split('.') {
        value = value.as_table_mut()?.remove(part)?;
    }
    SettingValue::from_toml(&value)
}

#[cfg(test)]
mod tests;
