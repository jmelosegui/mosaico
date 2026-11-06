//! Editing the `[[launch]]` tables of `keybindings.toml`, which start an
//! application or bring it into view, keeping what the user wrote around
//! them as the keybindings are kept.

use toml_edit::{Array, ArrayOfTables, Item, Table, Value};

use super::{KeybindingsDocument, combo, modifier_list, replace};
use crate::Result;
use crate::config::launch::read_launches;
use crate::config::store::file_error;
use crate::config::{LaunchBinding, Modifier, WindowMatch};

impl KeybindingsDocument {
    /// The launch bindings, in the order of the file, each read on its
    /// own: an entry the daemon could not read is an error here, at its
    /// place, so positions stay those of the file. One with a blank
    /// command is read, as it is being filled in; [`check`](Self::check)
    /// refuses it.
    pub fn launches(&self) -> Vec<Result<LaunchBinding>> {
        read_launches(&self.path, &self.doc.to_string()).unwrap_or_default()
    }

    /// The launch bindings, with one that cannot be read standing in as
    /// one without a key combination, which clashes with nothing.
    pub(super) fn launch_slots(&self) -> Vec<LaunchBinding> {
        let unbound = || LaunchBinding {
            command: String::new(),
            args: Vec::new(),
            key: String::new(),
            modifiers: Vec::new(),
            focus: false,
            windows: WindowMatch::default(),
        };
        let launches = self.launches().into_iter();
        launches.map(|l| l.unwrap_or_else(|_| unbound())).collect()
    }

    /// Adds `binding` after the other launch bindings. It is written as
    /// it is, so one being filled in can be added before it has a command
    /// or a key; [`check`](Self::check) refuses it until it has both.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Config`](crate::Error::Config) if the launch
    /// bindings are not written as tables.
    pub fn add_launch(&mut self, binding: &LaunchBinding) -> Result<()> {
        let launches = self.launches_mut()?;
        let mut table = Table::new();
        if !launches.is_empty() {
            table.decor_mut().set_prefix("\n");
        }
        write(&mut table, binding, true);
        launches.push(table);
        self.dirty = true;
        Ok(())
    }

    /// Writes what `binding` starts and which windows are its own, all but
    /// its key combination, into the launch binding at `index`, keeping
    /// the comments on its lines and the keys mosaico does not know.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Config`](crate::Error::Config) if there is no
    /// such launch binding or they are not written as tables.
    pub fn set_launch(&mut self, index: usize, binding: &LaunchBinding) -> Result<()> {
        write(self.launch_mut(index)?, binding, false);
        self.dirty = true;
        Ok(())
    }

    /// Binds the launch binding at `index` to `key` with `modifiers`, as
    /// [`set_key`](Self::set_key) binds a keybinding.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Config`](crate::Error::Config) if there is no
    /// such launch binding, they are not written as tables, or `key` is
    /// not a key name. The file is left as it was.
    pub fn set_launch_key(
        &mut self,
        index: usize,
        key: &str,
        modifiers: &[Modifier],
    ) -> Result<()> {
        let combo = combo(key, modifiers)?;
        let table = self.launch_mut(index)?;
        replace(table, "key", Value::from(combo.key.to_string()));
        replace(table, "modifiers", modifier_list(modifiers).into());
        self.dirty = true;
        Ok(())
    }

    /// Removes the launch binding at `index`, with the comments above it.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Config`](crate::Error::Config) if there is no
    /// such launch binding or they are not written as tables.
    pub fn remove_launch(&mut self, index: usize) -> Result<()> {
        self.launch_mut(index)?;
        self.launches_mut()?.remove(index);
        self.dirty = true;
        Ok(())
    }

    /// Checks every launch binding has a command and a key combination,
    /// as the daemon needs to bind it.
    pub(super) fn check_launches(&self) -> Result<()> {
        for (i, launch) in self.launches().into_iter().enumerate() {
            let launch = launch?;
            let n = i + 1;
            if launch.command.trim().is_empty() {
                let problem = format_args!("launch binding {n} has no command");
                return Err(file_error(&self.path, problem));
            }
            if launch.key.trim().is_empty() {
                let problem =
                    format_args!("launch binding {n} ({}) has no shortcut", launch.command);
                return Err(file_error(&self.path, problem));
            }
            launch
                .combo()
                .map_err(|e| file_error(&self.path, format_args!("launch binding {n}: {e}")))?;
        }
        Ok(())
    }

    fn launch_mut(&mut self, index: usize) -> Result<&mut Table> {
        let path = self.path.clone();
        let launches = self.launches_mut()?;
        let count = launches.len();
        launches
            .get_mut(index)
            .ok_or_else(|| file_error(&path, format_args!("no launch binding {index} of {count}")))
    }

    fn launches_mut(&mut self) -> Result<&mut ArrayOfTables> {
        let path = self.path.clone();
        let item = self
            .doc
            .entry("launch")
            .or_insert_with(|| Item::ArrayOfTables(ArrayOfTables::new()));
        item.as_array_of_tables_mut().ok_or_else(|| {
            let problem = "the launch bindings are not written as [[launch]] tables";
            file_error(&path, problem)
        })
    }
}

/// Writes the keys of `binding` into `table`, its key combination only
/// with `shortcut`, in place where they are there and after the others
/// where not, and drops the optional ones it does not have.
fn write(table: &mut Table, binding: &LaunchBinding, shortcut: bool) {
    replace(table, "command", Value::from(binding.command.as_str()));
    let args: Array = binding.args.iter().map(String::as_str).collect();
    optional(
        table,
        "args",
        (!binding.args.is_empty()).then(|| args.into()),
    );
    if shortcut {
        replace(table, "key", Value::from(binding.key.as_str()));
        replace(table, "modifiers", modifier_list(&binding.modifiers).into());
    }
    optional(table, "focus", binding.focus.then(|| true.into()));
    let windows = &binding.windows;
    for (name, value) in [
        ("match_class", &windows.match_class),
        ("match_title", &windows.match_title),
        ("match_path", &windows.match_path),
    ] {
        optional(table, name, value.as_deref().map(Value::from));
    }
}

fn optional(table: &mut Table, name: &str, value: Option<Value>) {
    match value {
        Some(value) => replace(table, name, value),
        None => {
            table.remove(name);
        }
    }
}

#[cfg(test)]
#[path = "tests/keybindings_document_launch_tests.rs"]
mod tests;
