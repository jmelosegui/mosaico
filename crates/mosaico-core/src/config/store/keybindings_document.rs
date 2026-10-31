//! Editing `keybindings.toml` one binding at a time, keeping what the
//! user wrote around it: comments, blank lines, the order of the
//! bindings, line endings and keys mosaico does not know.

use std::path::{Path, PathBuf};

use toml_edit::{Array, ArrayOfTables, DocumentMut, Item, Table, Value};

use super::merge_keybindings::keybinding_toml_entry;
use super::write_back::write_back;
use super::{ConfigStore, file_error, parse_toml};
use crate::config::rules::KeybindingsFile;
use crate::config::{Conflict, Keybinding, Modifier, conflicts, keybinding};
use crate::hotkey::{Key, KeyCombo};
use crate::{Action, Result};

/// `keybindings.toml` as written, open for editing.
///
/// Bindings are named by their position in the file, the one
/// [`bindings`](Self::bindings) lists them at. A key is checked as it is
/// set, by the parser the daemon uses, and the whole file is checked by
/// the loader before it is saved, so a save never leaves a file the
/// daemon cannot load. Two bindings on one key combination load, so
/// [`conflicts`](Self::conflicts) reports them for the caller to refuse.
#[derive(Debug)]
pub struct KeybindingsDocument {
    path: PathBuf,
    /// The text of the file when it was opened or last saved.
    original: String,
    doc: DocumentMut,
    dirty: bool,
}

impl ConfigStore {
    /// Opens `keybindings.toml` for editing. A missing file opens with the
    /// default bindings and is created by the first save.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Config`] naming the file if it cannot be read
    /// or the loader would not take it.
    pub fn open_keybindings(&self) -> Result<KeybindingsDocument> {
        KeybindingsDocument::open(&self.keybindings_path())
    }
}

impl KeybindingsDocument {
    fn open(path: &Path) -> Result<Self> {
        let (original, text) = match std::fs::read_to_string(path) {
            Ok(text) => (text.clone(), text),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let defaults = keybinding::defaults();
                let entries = defaults.iter().map(keybinding_toml_entry);
                (String::new(), entries.collect())
            }
            Err(e) => return Err(file_error(path, e)),
        };
        parse_toml::<KeybindingsFile>(path, &text)?;
        let doc = text.parse().map_err(|e| file_error(path, e))?;
        Ok(Self {
            path: path.to_path_buf(),
            original,
            doc,
            dirty: false,
        })
    }

    /// The bindings, as the loader reads them.
    pub fn bindings(&self) -> Vec<Keybinding> {
        let file: Result<KeybindingsFile> = parse_toml(&self.path, &self.doc.to_string());
        file.map(|f| f.keybinding).unwrap_or_default()
    }

    /// The key combinations more than one binding is bound to.
    pub fn conflicts(&self) -> Vec<Conflict> {
        conflicts(&self.bindings())
    }

    /// Binds the binding at `index` to `key` with `modifiers`, keeping any
    /// comment on its lines. The key is written the way the parser names
    /// it, whatever case it came in.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Config`] if there is no such binding, it is
    /// not written as a table, or `key` is not a key name. The file is
    /// left as it was.
    pub fn set_key(&mut self, index: usize, key: &str, modifiers: &[Modifier]) -> Result<()> {
        let combo = combo(key, modifiers)?;
        let path = self.path.clone();
        let table = self
            .bindings_mut()?
            .get_mut(index)
            .ok_or_else(|| file_error(&path, format_args!("no binding {index}")))?;
        replace(table, "key", Value::from(combo.key.to_string()));
        replace(table, "modifiers", modifier_list(modifiers).into());
        self.dirty = true;
        Ok(())
    }

    /// Adds a binding of `action` to `key` with `modifiers`, after the
    /// others.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Config`] if `key` is not a key name or the
    /// bindings are not written as tables. The file is left as it was.
    pub fn add(&mut self, action: Action, key: &str, modifiers: &[Modifier]) -> Result<()> {
        let combo = combo(key, modifiers)?;
        let bindings = self.bindings_mut()?;
        let mut table = Table::new();
        table.insert("action", Item::Value(action.to_string().into()));
        table.insert("key", Item::Value(combo.key.to_string().into()));
        table.insert("modifiers", Item::Value(modifier_list(modifiers).into()));
        if !bindings.is_empty() {
            table.decor_mut().set_prefix("\n");
        }
        bindings.push(table);
        self.dirty = true;
        Ok(())
    }

    /// Removes the binding at `index`, with the comments above it.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Config`] if there is no such binding or the
    /// bindings are not written as tables.
    pub fn remove(&mut self, index: usize) -> Result<()> {
        let path = self.path.clone();
        let bindings = self.bindings_mut()?;
        if index >= bindings.len() {
            return Err(file_error(&path, format_args!("no binding {index}")));
        }
        bindings.remove(index);
        self.dirty = true;
        Ok(())
    }

    /// Whether anything was changed since the file was opened or saved.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Checks the file, with the edits made so far, loads.
    ///
    /// # Errors
    ///
    /// Returns the [`Error::Config`] loading it would give.
    pub fn check(&self) -> Result<()> {
        parse_toml::<KeybindingsFile>(&self.path, &self.doc.to_string()).map(drop)
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
        let written = write_back(&self.path, &self.original, &edited)
            .map_err(|e| file_error(&self.path, e))?;
        self.original = written;
        self.dirty = false;
        Ok(())
    }

    fn bindings_mut(&mut self) -> Result<&mut ArrayOfTables> {
        let path = self.path.clone();
        let item = self
            .doc
            .entry("keybinding")
            .or_insert_with(|| Item::ArrayOfTables(ArrayOfTables::new()));
        item.as_array_of_tables_mut().ok_or_else(|| {
            let problem = "the bindings are not written as [[keybinding]] tables";
            file_error(&path, problem)
        })
    }
}

/// The combination of `key` and `modifiers`, checking the key name.
fn combo(key: &str, modifiers: &[Modifier]) -> Result<KeyCombo> {
    let key: Key = key.parse()?;
    Ok(KeyCombo {
        modifiers: modifiers.iter().copied().collect(),
        key,
    })
}

/// Sets `name` in `table`, keeping the comment around the old value.
fn replace(table: &mut Table, name: &str, mut new: Value) {
    if let Some(old) = table.get(name).and_then(Item::as_value) {
        *new.decor_mut() = old.decor().clone();
    }
    table.insert(name, Item::Value(new));
}

/// `modifiers` as the lowercase names of the file, each once, in the
/// order the defaults write them.
fn modifier_list(modifiers: &[Modifier]) -> Array {
    let names = [
        (Modifier::Alt, "alt"),
        (Modifier::Shift, "shift"),
        (Modifier::Ctrl, "ctrl"),
        (Modifier::Win, "win"),
    ];
    names
        .iter()
        .filter(|(m, _)| modifiers.contains(m))
        .map(|(_, name)| *name)
        .collect()
}

#[cfg(test)]
#[path = "tests/keybindings_document_tests.rs"]
mod tests;
