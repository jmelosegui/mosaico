//! The settings the window edits, and the edits not yet applied.

mod save;

pub(crate) use save::reason;
pub use save::{Loaded, save};

use std::collections::BTreeMap;

use mosaico_core::config::schema::{self, CONFIG_SETTINGS, SettingValue};
use mosaico_core::config::{SettingsDocument, Theme, ThemeConfig};

/// The settings the window shows: those of `config.toml`, by key, and
/// whether mosaico starts at sign-in, which no file holds.
#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    config: BTreeMap<&'static str, SettingValue>,
    /// Whether mosaico starts when the user signs in.
    pub start_at_sign_in: bool,
}

impl Settings {
    /// The settings in effect in `doc`, as loading it takes them.
    pub fn read(doc: &SettingsDocument, start_at_sign_in: bool) -> Self {
        Self::from_values(start_at_sign_in, |key| doc.value(key))
    }

    /// What a new install has.
    pub fn defaults() -> Self {
        Self::from_values(false, schema::config_default)
    }

    fn from_values(sign_in: bool, value: impl Fn(&str) -> Option<SettingValue>) -> Self {
        let config = CONFIG_SETTINGS
            .iter()
            .filter_map(|s| Some((s.key, value(s.key)?)))
            .collect();
        let mut settings = Self {
            config,
            start_at_sign_in: sign_in,
        };
        // Names are written the way the theme spells them, so a file
        // that capitalizes one does not count as edited.
        settings.set_theme(settings.theme());
        settings
    }

    /// The value of the setting of `config.toml` at `key`.
    ///
    /// # Panics
    ///
    /// If `key` is not a setting of `config.toml`.
    pub fn get(&self, key: &str) -> &SettingValue {
        self.config
            .get(key)
            .unwrap_or_else(|| panic!("no setting {key}"))
    }

    /// Sets the setting of `config.toml` at `key`. A key that is not one
    /// is ignored.
    pub fn set(&mut self, key: &str, value: SettingValue) {
        if let Some(slot) = self.config.get_mut(key) {
            *slot = value;
        }
    }

    /// The theme, from its name and flavor.
    pub fn theme(&self) -> Theme {
        let text = |key| self.get(key).as_text().unwrap_or_default().to_owned();
        ThemeConfig {
            name: text("theme.name"),
            flavor: text("theme.flavor"),
        }
        .resolve()
    }

    /// Sets the theme's name and flavor.
    pub fn set_theme(&mut self, theme: Theme) {
        self.set("theme.name", SettingValue::Text(theme.name().into()));
        let flavor = theme.flavor_name().into();
        self.set("theme.flavor", SettingValue::Text(flavor));
    }

    /// The settings of `config.toml` that differ from `other`.
    pub fn changed_keys<'a>(&'a self, other: &'a Self) -> impl Iterator<Item = &'static str> + 'a {
        self.config
            .iter()
            .filter(|(key, value)| other.config.get(*key) != Some(value))
            .map(|(key, _)| *key)
    }

    /// How many settings differ from `other`. The theme and its flavor
    /// count apart.
    pub fn changes_from(&self, other: &Self) -> usize {
        let sign_in = self.start_at_sign_in != other.start_at_sign_in;
        self.changed_keys(other).count() + usize::from(sign_in)
    }
}

/// Settings as they were loaded, and as they are being edited.
#[derive(Debug, Clone)]
pub struct Draft<T> {
    saved: T,
    /// The settings with the edits made so far.
    pub edit: T,
}

impl<T: Clone> Draft<T> {
    /// A draft with no edits.
    pub fn new(saved: T) -> Self {
        Self {
            edit: saved.clone(),
            saved,
        }
    }

    /// The settings as they were loaded.
    pub fn saved(&self) -> &T {
        &self.saved
    }

    /// Drops every edit.
    pub fn revert(&mut self) {
        self.edit = self.saved.clone();
    }
}

impl Draft<Settings> {
    /// How many settings were edited.
    pub fn changes(&self) -> usize {
        self.edit.changes_from(&self.saved)
    }
}

#[cfg(test)]
mod tests;
