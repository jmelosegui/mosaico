//! The settings of `bar.toml` the page edits, read from the file and
//! written back to it.

use std::collections::BTreeMap;

use mosaico_core::Error;
use mosaico_core::config::schema::{BAR_SETTINGS, SettingValue, bar_default};
use mosaico_core::config::{BarWidgets, ConfigStore, SettingsDocument};

use crate::model::reason;

/// What the Bar page shows: the single settings of the file, by key, and
/// which widgets are on each side.
#[derive(Debug, Clone, PartialEq)]
pub struct BarState {
    values: BTreeMap<&'static str, SettingValue>,
    pub widgets: BarWidgets,
}

impl BarState {
    /// What a new install has.
    pub fn defaults() -> Self {
        Self {
            values: BAR_SETTINGS
                .iter()
                .filter_map(|s| Some((s.key, bar_default(s.key)?)))
                .collect(),
            widgets: BarWidgets::default(),
        }
    }

    /// What is in effect in `doc`, as loading it takes it.
    ///
    /// # Errors
    ///
    /// If the file does not load.
    pub fn read(doc: &SettingsDocument) -> Result<Self, Error> {
        doc.check()?;
        let mut state = Self::defaults();
        for (key, slot) in &mut state.values {
            if let Some(value) = doc.value(key) {
                *slot = value;
            }
        }
        state.widgets = doc.widgets()?;
        Ok(state)
    }

    /// The setting at `key`.
    pub fn get(&self, key: &str) -> Option<&SettingValue> {
        self.values.get(key)
    }

    /// Sets the setting at `key`. A key that is not one is ignored.
    pub fn set(&mut self, key: &str, value: SettingValue) {
        if let Some(slot) = self.values.get_mut(key) {
            *slot = value;
        }
    }

    /// The settings that differ from `other`.
    fn changed_keys<'a>(&'a self, other: &'a Self) -> impl Iterator<Item = &'static str> + 'a {
        self.values
            .iter()
            .filter(|(key, value)| other.values.get(*key) != Some(value))
            .map(|(key, _)| *key)
    }

    /// How many edits there are from `other`: the settings that differ,
    /// and one for the widgets when they do.
    pub fn changes_from(&self, other: &Self) -> usize {
        self.changed_keys(other).count() + usize::from(self.widgets != other.widgets)
    }
}

/// The state of `bar.toml` in `store`. A missing file has the defaults
/// and can be edited; the first save creates it.
///
/// # Errors
///
/// Says why the file cannot be edited: it cannot be read, or the daemon
/// would not load it.
pub fn load(store: &ConfigStore) -> Result<BarState, String> {
    let doc = store.open_bar_settings().map_err(reason)?;
    BarState::read(&doc).map_err(reason)
}

/// Writes what `edit` changes from `saved` to `bar.toml` in `store`,
/// leaving the rest of it as it is.
///
/// # Errors
///
/// Says why the file was not written. It is then left as it was.
pub fn save(store: &ConfigStore, saved: &BarState, edit: &BarState) -> Result<(), String> {
    let mut doc = store.open_bar_settings().map_err(reason)?;
    for key in edit.changed_keys(saved) {
        if let Some(value) = edit.get(key) {
            doc.put(key, value).map_err(reason)?;
        }
    }
    if edit.widgets != saved.widgets {
        doc.set_widgets(&edit.widgets).map_err(reason)?;
    }
    if doc.is_dirty() {
        doc.save().map_err(reason)?;
    }
    Ok(())
}
