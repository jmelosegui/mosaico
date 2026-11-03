//! Launch bindings: key combinations that start an application, or bring
//! an open window of it into view.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::hotkey::KeyCombo;
use crate::{Error, Result};

use super::keybinding::{self, Modifier};
use super::rules;

/// A `[[launch]]` entry of the keybindings file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaunchBinding {
    /// The program to start, as the user would type it in the system's
    /// run box: a name the system finds, a full path, or a URI.
    pub command: String,
    /// Arguments passed to the program, one per entry.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    /// Key name, checked by [`LaunchBinding::combo`] as for keybindings.
    pub key: String,
    /// Modifier keys.
    pub modifiers: Vec<Modifier>,
    /// Focus an open window of the application, when there is one,
    /// instead of starting another.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub focus: bool,
    /// Which windows count as the application's.
    #[serde(flatten)]
    pub windows: WindowMatch,
}

impl LaunchBinding {
    /// The key combination that triggers this binding.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Config`](crate::Error::Config) when the key name
    /// is not one mosaico knows.
    pub fn combo(&self) -> Result<KeyCombo> {
        keybinding::combo_of(&self.key, &self.modifiers)
    }

    /// Whether a window with this class, title and executable path
    /// belongs to the application this binding starts.
    ///
    /// The `match_*` fields mean what they mean in a window rule. With
    /// none of them, a window belongs to it when the file name of its
    /// executable, without extension, is the file name of `command`
    /// without extension, ignoring case.
    pub fn matches(&self, class: &str, title: &str, path: &str) -> bool {
        if self.windows.is_empty() {
            let stem = |path: &str| {
                Path::new(&path.replace('\\', "/"))
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_ascii_lowercase())
            };
            return stem(path).is_some_and(|window| Some(window) == stem(&self.command));
        }
        rules::matches(
            class,
            title,
            path,
            self.windows.match_class.as_deref(),
            self.windows.match_title.as_deref(),
            self.windows.match_path.as_deref(),
        )
    }

    /// What names the binding in messages: its command.
    pub fn name(&self) -> String {
        format!("launch {}", self.command)
    }
}

/// Which windows belong to an application, with the meaning of the
/// fields of a window rule.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowMatch {
    /// Windows with this exact class name (case-insensitive).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub match_class: Option<String>,
    /// Windows whose title contains this (case-insensitive).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub match_title: Option<String>,
    /// Windows whose executable path ends with this (case-insensitive, on
    /// a path component boundary).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub match_path: Option<String>,
}

impl WindowMatch {
    /// Whether no field is given.
    pub fn is_empty(&self) -> bool {
        self.match_class.is_none() && self.match_title.is_none() && self.match_path.is_none()
    }
}

/// The `[[launch]]` tables of a keybindings file, each read on its own.
#[derive(Deserialize)]
struct LaunchTables {
    #[serde(default)]
    launch: Vec<toml::Table>,
}

/// Reads the launch bindings in `content`, the text of the keybindings
/// file at `path`.
///
/// Each entry is read on its own, so a mistake in one puts only that one
/// out of action. An entry fails when a field is missing or of the wrong
/// type, or its command is blank.
///
/// # Errors
///
/// Returns [`Error::Config`](crate::Error::Config) naming `path` when
/// `content` is not TOML or `launch` is not a list of tables.
pub fn parse_launches(path: &Path, content: &str) -> Result<Vec<Result<LaunchBinding>>> {
    let error = |problem: String| Error::config(Some(PathBuf::from(path)), problem);
    let tables: LaunchTables = toml::from_str(content).map_err(|e| error(e.to_string()))?;
    Ok(tables
        .launch
        .into_iter()
        .enumerate()
        .map(|(i, table)| {
            parse_one(table)
                .map_err(|problem| error(format!("launch binding {}: {problem}", i + 1)))
        })
        .collect())
}

fn parse_one(table: toml::Table) -> std::result::Result<LaunchBinding, String> {
    let binding: LaunchBinding = table
        .try_into()
        .map_err(|e: toml::de::Error| e.message().to_string())?;
    if binding.command.trim().is_empty() {
        return Err("`command` is empty".to_string());
    }
    Ok(binding)
}

#[cfg(test)]
#[path = "launch_tests.rs"]
mod tests;
