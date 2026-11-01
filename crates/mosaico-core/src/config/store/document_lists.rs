//! Editing the lists of `bar.toml` that are not single settings: the
//! monitors the bar is on and the mouse bindings of its widgets.

use toml_edit::{Array, ArrayOfTables, Item, Table, TableLike, value};

use super::{SettingsDocument, tables_for};
use crate::Result;
use crate::config::{BarConfig, ClickBinding};

impl SettingsDocument {
    /// The monitors the bar is on, by number from 0, in effect: none
    /// for every monitor, as loading the file takes it.
    ///
    /// # Errors
    ///
    /// Returns the [`Error::Config`](crate::Error::Config) loading the
    /// file would give.
    pub fn monitors(&self) -> Result<Vec<usize>> {
        Ok(self.bar_config()?.monitors)
    }

    /// Makes the file put the bar on `monitors`, none meaning every
    /// monitor. With none the key is removed, so it follows the default.
    ///
    /// # Errors
    ///
    /// Returns the [`Error::Config`](crate::Error::Config) loading the
    /// file would give. The file is left as it was.
    pub fn set_monitors(&mut self, monitors: &[usize]) -> Result<()> {
        if self.monitors()? == monitors {
            return Ok(());
        }
        if monitors.is_empty() {
            self.doc.remove("monitors");
        } else {
            let numbers: Array = monitors.iter().map(|&m| m as i64).collect();
            self.doc["monitors"] = value(numbers);
        }
        self.dirty = true;
        Ok(())
    }

    /// The bindings of clicks on the widgets in effect, the defaults
    /// with the file's `[[mouse.click]]` entries applied, see
    /// [`ClickBinding::effective`].
    ///
    /// # Errors
    ///
    /// Returns the [`Error::Config`](crate::Error::Config) loading the
    /// file would give.
    pub fn clicks(&self) -> Result<Vec<ClickBinding>> {
        Ok(ClickBinding::effective(&self.bar_config()?.mouse.click))
    }

    /// Makes the file give the clicks the bindings in `clicks`, by
    /// listing only what differs from the defaults, see
    /// [`ClickBinding::entries_for`]. The bindings of the defaults alone
    /// leave no entries in the file.
    ///
    /// # Errors
    ///
    /// Returns the [`Error::Config`](crate::Error::Config) loading the
    /// file would give. The file is left as it was.
    pub fn set_clicks(&mut self, clicks: &[ClickBinding]) -> Result<()> {
        let entries = ClickBinding::entries_for(clicks);
        if self.clicks()? == ClickBinding::effective(&entries) {
            return Ok(());
        }
        let Some(mouse) = tables_for(self.doc.as_table_mut(), &["mouse"]) else {
            return Err(self.error("mouse has no table to go in"));
        };
        replace_clicks(mouse, &entries);
        self.dirty = true;
        Ok(())
    }

    fn bar_config(&self) -> Result<BarConfig> {
        super::parse_toml(&self.path, &self.doc.to_string())
    }
}

fn replace_clicks(mouse: &mut dyn TableLike, entries: &[ClickBinding]) {
    if entries.is_empty() {
        mouse.remove("click");
        return;
    }
    let mut list = ArrayOfTables::new();
    for entry in entries {
        let mut table = Table::new();
        let keys = [
            ("widget", &entry.widget),
            ("button", &entry.button),
            ("action", &entry.action),
        ];
        for (key, text) in keys {
            if let Some(text) = text {
                table.insert(key, value(text.as_str()));
            }
        }
        list.push(table);
    }
    mouse.insert("click", Item::ArrayOfTables(list));
}
