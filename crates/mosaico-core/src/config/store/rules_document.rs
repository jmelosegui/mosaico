//! Editing `user-rules.toml` one rule at a time, keeping what the user
//! wrote around the rules: comments, blank lines, line endings and keys
//! mosaico does not know.

use std::path::{Path, PathBuf};

use toml_edit::{ArrayOfTables, DocumentMut, Item, Table, Value};

use super::write_back::write_back;
use super::{ConfigStore, file_error, parse_toml};
use crate::Result;
use crate::config::WindowRule;
use crate::config::rules::UserRulesFile;

/// A rule of the file, as it is to be written: the rule, and the rule of
/// the file it comes from, whose comments and unknown keys it keeps.
pub type RuleSlot = (Option<usize>, WindowRule);

/// `user-rules.toml` as written, open for editing.
///
/// The rules are in the order the file lists them, which is the order
/// they match windows in. Rules are checked as they are put in, and the
/// whole file is checked by the loader before it is saved, so a save
/// never leaves a file the daemon cannot load.
#[derive(Debug)]
pub struct RulesDocument {
    path: PathBuf,
    /// The text of the file when it was opened or last saved.
    original: String,
    doc: DocumentMut,
    dirty: bool,
}

impl ConfigStore {
    /// Opens `user-rules.toml` for editing. A missing file opens with no
    /// rules and is created by the first save.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Config`](crate::Error::Config) naming the file
    /// if it cannot be read or the loader would refuse it.
    pub fn open_user_rules(&self) -> Result<RulesDocument> {
        RulesDocument::open(&self.user_rules_path())
    }
}

impl WindowRule {
    /// Whether the rule can be written to a rules file: it matches on
    /// something, and not on an empty class or path, which no window has.
    ///
    /// # Errors
    ///
    /// Returns what is wrong with the rule.
    pub fn check(&self) -> std::result::Result<(), String> {
        let blank = |text: &Option<String>| text.as_ref().is_some_and(|t| t.trim().is_empty());
        if self.match_class.is_none() && self.match_title.is_none() && self.match_path.is_none() {
            Err("a rule needs a class, a title or a path to match".into())
        } else if blank(&self.match_class) {
            Err("the class to match is empty".into())
        } else if blank(&self.match_path) {
            Err("the path to match is empty".into())
        } else {
            Ok(())
        }
    }
}

impl RulesDocument {
    fn open(path: &Path) -> Result<Self> {
        let original = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(e) => return Err(file_error(path, e)),
        };
        let doc = original.parse().map_err(|e| file_error(path, e))?;
        let document = Self {
            path: path.to_path_buf(),
            original,
            doc,
            dirty: false,
        };
        document.check()?;
        Ok(document)
    }

    /// The rules, in the order of the file.
    pub fn rules(&self) -> Vec<WindowRule> {
        parse_toml::<UserRulesFile>(&self.path, &self.doc.to_string())
            .map(|file| file.rule)
            .unwrap_or_default()
    }

    /// Writes the rules as `slots` lay them out: rules that come from one
    /// of the file keep their comments and the keys they hold that
    /// mosaico does not know, wherever they are moved to; rules that
    /// are not in `slots` are removed, and those without an origin added.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Config`](crate::Error::Config) if a rule
    /// fails [`WindowRule::check`]. The file is left as it was.
    pub fn arrange(&mut self, slots: &[RuleSlot]) -> Result<()> {
        for (_, rule) in slots {
            rule.check().map_err(|e| file_error(&self.path, e))?;
        }
        let mut old = self.tables();
        let before = self.rules();
        let text = self.doc.to_string();
        let positions: Vec<usize> = old.iter().filter_map(Table::position).collect();
        let header = old.first_mut().map(take_header).unwrap_or_default();
        let mut new = ArrayOfTables::new();
        for (k, (origin, rule)) in slots.iter().enumerate() {
            let source = origin.and_then(|i| old.get(i).zip(before.get(i)));
            let mut table = match source {
                Some((table, was)) if was == rule => table.clone(),
                Some((table, _)) => written(table.clone(), rule),
                None => written(Table::new(), rule),
            };
            if origin.is_some() && *origin != Some(k) {
                separate(&mut table, k == 0);
            }
            if k == 0 {
                prepend(&mut table, &header);
            }
            if let Some(last) = positions.last() {
                table.set_position(
                    positions
                        .get(k)
                        .copied()
                        .unwrap_or(last + k + 1 - positions.len()),
                );
            }
            new.push(table);
        }
        self.doc.insert("rule", Item::ArrayOfTables(new));
        self.dirty |= self.doc.to_string() != text;
        Ok(())
    }

    /// Adds `rule` after the others.
    ///
    /// # Errors
    ///
    /// As [`arrange`](Self::arrange).
    pub fn add(&mut self, rule: WindowRule) -> Result<()> {
        let mut slots = self.slots();
        slots.push((None, rule));
        self.arrange(&slots)
    }

    /// Replaces the rule at `index` with `rule`, changing only the keys
    /// that differ.
    ///
    /// # Errors
    ///
    /// As [`arrange`](Self::arrange), or if there is no rule at `index`.
    pub fn update(&mut self, index: usize, rule: WindowRule) -> Result<()> {
        let mut slots = self.slots();
        let slot = self.slot_mut(&mut slots, index)?;
        slot.1 = rule;
        self.arrange(&slots)
    }

    /// Removes the rule at `index`, with the comments above it.
    ///
    /// # Errors
    ///
    /// As [`arrange`](Self::arrange), or if there is no rule at `index`.
    pub fn remove(&mut self, index: usize) -> Result<()> {
        let mut slots = self.slots();
        self.slot_mut(&mut slots, index)?;
        slots.remove(index);
        self.arrange(&slots)
    }

    /// Moves the rule at `index` one place up, so it matches before the
    /// rule above it. The first rule stays.
    ///
    /// # Errors
    ///
    /// As [`arrange`](Self::arrange), or if there is no rule at `index`.
    pub fn move_up(&mut self, index: usize) -> Result<()> {
        let mut slots = self.slots();
        self.slot_mut(&mut slots, index)?;
        if index > 0 {
            slots.swap(index, index - 1);
        }
        self.arrange(&slots)
    }

    /// Moves the rule at `index` one place down. The last rule stays.
    ///
    /// # Errors
    ///
    /// As [`arrange`](Self::arrange), or if there is no rule at `index`.
    pub fn move_down(&mut self, index: usize) -> Result<()> {
        let mut slots = self.slots();
        self.slot_mut(&mut slots, index)?;
        if index + 1 < slots.len() {
            slots.swap(index, index + 1);
        }
        self.arrange(&slots)
    }

    /// Whether anything changed since the file was opened or saved.
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
    /// Returns the [`Error::Config`](crate::Error::Config) loading it
    /// would give.
    pub fn check(&self) -> Result<()> {
        parse_toml::<UserRulesFile>(&self.path, &self.doc.to_string()).map(drop)
    }

    /// Writes the edits to the file, in its own line endings, after
    /// checking the result loads. A missing file is created.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Config`](crate::Error::Config) naming the file
    /// if the edited file would not load, or cannot be written. The file
    /// is left as it was.
    pub fn save(&mut self) -> Result<()> {
        self.check()?;
        let edited = self.doc.to_string();
        let written = write_back(&self.path, &self.original, &edited)
            .map_err(|e| file_error(&self.path, e))?;
        self.original = written;
        self.dirty = false;
        Ok(())
    }

    fn slots(&self) -> Vec<RuleSlot> {
        let rules = self.rules();
        rules
            .into_iter()
            .enumerate()
            .map(|(i, r)| (Some(i), r))
            .collect()
    }

    fn slot_mut<'a>(&self, slots: &'a mut [RuleSlot], index: usize) -> Result<&'a mut RuleSlot> {
        let count = slots.len();
        slots
            .get_mut(index)
            .ok_or_else(|| file_error(&self.path, format!("no rule {index} of {count}")))
    }

    /// The tables of the `[[rule]]` entries, whether written as an array
    /// of tables or inline.
    fn tables(&self) -> Vec<Table> {
        match self.doc.get("rule") {
            Some(Item::ArrayOfTables(tables)) => tables.iter().cloned().collect(),
            Some(Item::Value(Value::Array(rules))) => rules
                .iter()
                .filter_map(|rule| rule.as_inline_table())
                .map(|rule| rule.clone().into_table())
                .collect(),
            _ => Vec::new(),
        }
    }
}

/// `table` with the keys of `rule`: what the rule matches on and its
/// action, in place where they are there and after the others where not.
fn written(mut table: Table, rule: &WindowRule) -> Table {
    put(&mut table, "match_class", rule.match_class.as_deref());
    put(&mut table, "match_title", rule.match_title.as_deref());
    put(&mut table, "match_path", rule.match_path.as_deref());
    put(&mut table, "action", Some(rule.action.name()));
    table.remove("manage");
    table
}

/// Takes the comments at the top of the file, up to the last blank line
/// before the first rule, off `table`, so they stay at the top when the
/// first rule moves.
fn take_header(table: &mut Table) -> String {
    let prefix = prefix(table);
    let end = prefix
        .split_inclusive('\n')
        .scan(0, |end, line| {
            *end += line.len();
            Some((*end, line.trim().is_empty()))
        })
        .filter(|(_, blank)| *blank)
        .map(|(end, _)| end)
        .last()
        .unwrap_or(0);
    table.decor_mut().set_prefix(prefix[end..].to_owned());
    prefix[..end].to_owned()
}

fn prefix(table: &Table) -> String {
    let raw = table.decor().prefix();
    raw.and_then(|p| p.as_str()).unwrap_or_default().to_owned()
}

fn prepend(table: &mut Table, header: &str) {
    if !header.is_empty() {
        let prefix = format!("{header}{}", prefix(table));
        table.decor_mut().set_prefix(prefix);
    }
}

/// Gives a rule that moved a blank line above its comments, or none when
/// it is now the first.
fn separate(table: &mut Table, first: bool) {
    let prefix = prefix(table);
    let comments = prefix.trim_start_matches(['\r', '\n']);
    let blank = if first { "" } else { "\n" };
    table.decor_mut().set_prefix(format!("{blank}{comments}"));
}

fn put(table: &mut Table, key: &str, text: Option<&str>) {
    let Some(text) = text else {
        table.remove(key);
        return;
    };
    let mut new = Value::from(text);
    match table.get_mut(key) {
        Some(Item::Value(old)) => {
            *new.decor_mut() = old.decor().clone();
            *old = new;
        }
        _ => {
            table.insert(key, Item::Value(new));
        }
    }
}

#[cfg(test)]
#[path = "tests/rules_document_tests.rs"]
mod tests;
