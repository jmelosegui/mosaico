//! What is wrong with the values written in a config file, said with the
//! names and ranges of the settings, so `mosaico doctor` and the settings
//! window describe a value the same way.

use std::fmt;

use super::{SettingDescriptor, SettingKind, SettingValue};

/// One value of a file that mosaico does not take as written.
#[derive(Debug, Clone, PartialEq)]
pub enum SettingProblem {
    /// A number outside the setting's range. Loading brings it into the
    /// range.
    OutOfRange {
        /// The setting.
        setting: &'static SettingDescriptor,
        /// What the file says.
        value: SettingValue,
    },
    /// A value of another kind, or a name the setting does not list.
    Refused {
        /// The setting.
        setting: &'static SettingDescriptor,
        /// Why it is refused.
        reason: String,
    },
    /// A key no setting has: a typo, or a setting of another version.
    Unknown {
        /// The key, with its tables joined by dots.
        key: String,
    },
}

impl fmt::Display for SettingProblem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfRange { setting, value } => {
                write!(
                    f,
                    "{} ({}) is {value}; it takes ",
                    setting.label, setting.key
                )?;
                match setting.kind {
                    SettingKind::Int { min, max, unit: "" } => {
                        write!(f, "{min} to {max}")
                    }
                    SettingKind::Int { min, max, unit } => write!(f, "{min} to {max} {unit}"),
                    SettingKind::Float { min, max, .. } => write!(f, "{min} to {max}"),
                    _ => f.write_str("another value"),
                }
            }
            Self::Refused { setting, reason } => write!(f, "{}: {reason}", setting.label),
            Self::Unknown { key } => write!(f, "{key} is not a setting mosaico knows"),
        }
    }
}

/// The problems of the single values in `text`, a TOML file whose
/// settings are `settings`. Lists are left out: they are checked by the
/// file's own loader. A file that does not parse has none here.
pub fn review(text: &str, settings: &'static [SettingDescriptor]) -> Vec<SettingProblem> {
    let Ok(table) = text.parse::<toml::Table>() else {
        return Vec::new();
    };
    let mut problems = Vec::new();
    walk("", &table, settings, &mut problems);
    problems
}

fn walk(
    prefix: &str,
    table: &toml::Table,
    settings: &'static [SettingDescriptor],
    problems: &mut Vec<SettingProblem>,
) {
    for (name, value) in table {
        let key = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}.{name}")
        };
        if let toml::Value::Table(inner) = value {
            walk(&key, inner, settings, problems);
            continue;
        }
        let Some(value) = SettingValue::from_toml(value) else {
            continue;
        };
        match settings.iter().find(|s| s.key == key) {
            Some(setting) => problems.extend(problem(setting, value)),
            None => problems.push(SettingProblem::Unknown { key }),
        }
    }
}

fn problem(setting: &'static SettingDescriptor, value: SettingValue) -> Option<SettingProblem> {
    let Some(used) = setting.normalize(value.clone()) else {
        let reason = match setting.check(&value) {
            Err(reason) => reason,
            Ok(()) => format!("{} does not take {value}", setting.key),
        };
        return Some(SettingProblem::Refused { setting, reason });
    };
    let same = match (&used, &value) {
        (SettingValue::Float(x), SettingValue::Int(n)) => *x == *n as f64,
        (used, value) => used == value,
    };
    (!same).then_some(SettingProblem::OutOfRange { setting, value })
}

#[cfg(test)]
#[path = "review_tests.rs"]
mod tests;
