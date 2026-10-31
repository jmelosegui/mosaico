//! Editing `config.toml` or `bar.toml` one setting at a time, keeping what the user
//! wrote around it: comments, blank lines, the order of the tables, line
//! endings and keys mosaico does not know.

use std::path::{Path, PathBuf};

use toml_edit::{DocumentMut, Item, Table, TableLike, Value};

use super::write_back::write_back;
use super::{ConfigStore, file_error, parse_toml};
use crate::config::schema::{self, SettingDescriptor, SettingValue};
use crate::config::{BarConfig, Config};
use crate::{Error, Result};

#[path = "document_widgets.rs"]
mod widgets;

/// `config.toml` or `bar.toml` as written, open for editing.
///
/// Only what is set or reset changes; the rest of the file is written
/// back as it was. A setting is checked against its
/// [descriptor](schema::SettingDescriptor) as it is set, and the whole
/// file is checked by the loader before it is saved, so a save never
/// leaves a file the daemon cannot load.
#[derive(Debug)]
pub struct SettingsDocument {
    path: PathBuf,
    kind: Kind,
    /// The text of the file when it was opened or last saved.
    original: String,
    doc: DocumentMut,
    dirty: bool,
}

impl ConfigStore {
    /// Opens `config.toml` for editing. A missing file opens empty and is
    /// created by the first save.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Config`] naming the file if it cannot be read
    /// or is not TOML.
    pub fn open_settings(&self) -> Result<SettingsDocument> {
        SettingsDocument::open(&self.config_path(), CONFIG)
    }

    /// Opens `bar.toml` for editing, as [`open_settings`](Self::open_settings)
    /// does `config.toml`.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Config`] naming the file if it cannot be read
    /// or is not TOML.
    pub fn open_bar_settings(&self) -> Result<SettingsDocument> {
        SettingsDocument::open(&self.bar_path(), BAR)
    }
}

/// What differs between the files a document edits.
#[derive(Debug, Clone, Copy)]
struct Kind {
    settings: &'static [SettingDescriptor],
    default: fn(&str) -> Option<SettingValue>,
    /// Parses the text as the loader of the file does.
    check: fn(&Path, &str) -> Result<()>,
}

const CONFIG: Kind = Kind {
    settings: schema::CONFIG_SETTINGS,
    default: schema::config_default,
    check: |path, text| parse_toml::<Config>(path, text).map(drop),
};

const BAR: Kind = Kind {
    settings: schema::BAR_SETTINGS,
    default: schema::bar_default,
    check: |path, text| parse_toml::<BarConfig>(path, text).map(drop),
};

impl SettingsDocument {
    fn open(path: &Path, kind: Kind) -> Result<Self> {
        let original = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(e) => return Err(file_error(path, e)),
        };
        let doc = original.parse().map_err(|e| file_error(path, e))?;
        Ok(Self {
            path: path.to_path_buf(),
            kind,
            original,
            doc,
            dirty: false,
        })
    }

    /// What the file sets `key` to, or `None` where it leaves it out or
    /// holds a table or a list there.
    pub fn get(&self, key: &str) -> Option<SettingValue> {
        let (tables, name) = key.rsplit_once('.').unwrap_or(("", key));
        let mut table: &dyn TableLike = self.doc.as_table();
        for part in tables.split('.').filter(|p| !p.is_empty()) {
            table = table.get(part)?.as_table_like()?;
        }
        from_value(table.get(name)?.as_value()?)
    }

    /// What `key` is in effect: what the file sets it to, as loading
    /// takes it, or its default where the file leaves it out or holds a
    /// value it does not take.
    pub fn value(&self, key: &str) -> Option<SettingValue> {
        let setting = self.setting(key)?;
        self.get(key)
            .and_then(|v| setting.normalize(v))
            .or_else(|| (self.kind.default)(key))
    }

    /// Sets `key` to `value`, or removes it where `value` is its default,
    /// so the file holds only what the user chose and follows the
    /// defaults of later releases for the rest.
    ///
    /// # Errors
    ///
    /// As [`set`](Self::set).
    pub fn put(&mut self, key: &str, value: &SettingValue) -> Result<()> {
        if (self.kind.default)(key).as_ref() == Some(value) {
            self.reset(key);
            Ok(())
        } else {
            self.set(key, value)
        }
    }

    /// Sets `key` to `value`, keeping any comment on its line, and adding
    /// the tables it is in when the file has none.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Config`] if `key` is not a setting of the file
    /// or `value` is not one it takes. The file is left as it was.
    pub fn set(&mut self, key: &str, value: &SettingValue) -> Result<()> {
        let setting = self
            .setting(key)
            .ok_or_else(|| self.error(format!("no setting {key}")))?;
        setting.check(value).map_err(|e| self.error(e))?;
        if self.get(key).as_ref() == Some(value) {
            return Ok(());
        }
        let (tables, name) = setting.path();
        let Some(table) = tables_for(self.doc.as_table_mut(), &tables) else {
            return Err(self.error(format!("{key} has no table to go in")));
        };
        let mut new = to_value(value);
        if let Some(old) = table.get(name).and_then(Item::as_value) {
            *new.decor_mut() = old.decor().clone();
        }
        table.insert(name, Item::Value(new));
        self.dirty = true;
        Ok(())
    }

    /// Removes `key` from the file, so it takes its default. The tables
    /// it was in stay, with their comments.
    pub fn reset(&mut self, key: &str) {
        let (tables, name) = key.rsplit_once('.').unwrap_or(("", key));
        let mut table: &mut dyn TableLike = self.doc.as_table_mut();
        for part in tables.split('.').filter(|p| !p.is_empty()) {
            match table.get_mut(part).and_then(Item::as_table_like_mut) {
                Some(inner) => table = inner,
                None => return,
            }
        }
        if table.remove(name).is_some() {
            self.dirty = true;
        }
    }

    /// Whether anything was set or reset since the file was opened or
    /// saved.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Whether the file no longer holds what it held when it was opened
    /// or saved, because something else wrote it.
    pub fn has_changed_on_disk(&self) -> bool {
        let now = std::fs::read_to_string(&self.path).unwrap_or_default();
        now != self.original
    }

    /// Checks the file, with the edits made so far, loads.
    ///
    /// # Errors
    ///
    /// Returns the [`Error::Config`] loading it would give.
    pub fn check(&self) -> Result<()> {
        (self.kind.check)(&self.path, &self.doc.to_string())
    }

    /// Writes the edits to the file, in its own line endings, after
    /// checking the result loads.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Config`] naming the file if the edited file
    /// would not load, or cannot be written. The file is left as it was.
    pub fn save(&mut self) -> Result<()> {
        self.check()?;
        let edited = self.doc.to_string();
        let written = write_back(&self.path, &self.original, &edited).map_err(|e| self.error(e))?;
        self.original = written;
        self.dirty = false;
        Ok(())
    }

    fn setting(&self, key: &str) -> Option<&'static SettingDescriptor> {
        self.kind.settings.iter().find(|s| s.key == key)
    }

    fn error(&self, problem: impl std::fmt::Display) -> Error {
        file_error(&self.path, problem)
    }
}

/// The table at `path` under `root`, adding any of it that is missing.
fn tables_for<'a>(root: &'a mut Table, path: &[&str]) -> Option<&'a mut dyn TableLike> {
    let mut table: &mut dyn TableLike = root;
    for part in path {
        let item = table.entry(part).or_insert_with(|| {
            let mut new = Table::new();
            new.set_implicit(true);
            Item::Table(new)
        });
        if item.as_table_like().is_none() {
            *item = Item::Table(Table::new());
        }
        table = item.as_table_like_mut()?;
    }
    Some(table)
}

fn from_value(value: &Value) -> Option<SettingValue> {
    match value {
        Value::Boolean(b) => Some(SettingValue::Bool(*b.value())),
        Value::Integer(n) => Some(SettingValue::Int(*n.value())),
        Value::Float(x) => Some(SettingValue::Float(*x.value())),
        Value::String(s) => Some(SettingValue::Text(s.value().clone())),
        _ => None,
    }
}

fn to_value(value: &SettingValue) -> Value {
    match value {
        SettingValue::Bool(b) => Value::from(*b),
        SettingValue::Int(n) => Value::from(*n),
        SettingValue::Float(x) => Value::from(*x),
        SettingValue::Text(s) => Value::from(s.as_str()),
    }
}

#[cfg(test)]
#[path = "tests/document_bar_tests.rs"]
mod bar_tests;
#[cfg(test)]
#[path = "tests/document_tests.rs"]
mod tests;
