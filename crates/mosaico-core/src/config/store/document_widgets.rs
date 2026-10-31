//! Editing which widgets `bar.toml` lists, and in what order.
//!
//! A widget keeps everything written in its own table, comments
//! included, wherever it moves to.

use std::str::FromStr;

use serde::Serialize;
use toml_edit::{Array, ArrayOfTables, DocumentMut, Item, Table, Value, value};

use super::SettingsDocument;
use crate::Result;
use crate::config::{BarConfig, BarWidgets, BarZone, WidgetConfig, WidgetKind};

/// A widget as its table in the file.
struct Entry {
    kind: WidgetKind,
    table: Table,
    used: bool,
}

#[derive(Serialize)]
struct Side<'a> {
    left: Option<&'a [WidgetConfig]>,
    right: Option<&'a [WidgetConfig]>,
}

impl SettingsDocument {
    /// The widgets in effect, as loading the file takes them: those it
    /// lists, or the defaults of a side it leaves out.
    ///
    /// # Errors
    ///
    /// Returns the [`Error::Config`](crate::Error::Config) loading the
    /// file would give.
    pub fn widgets(&self) -> Result<BarWidgets> {
        let config: BarConfig = super::parse_toml(&self.path, &self.doc.to_string())?;
        Ok(BarWidgets::of(&config))
    }

    /// Makes the file list `widgets`: which are on each side, in what
    /// order, and whether they are enabled. A widget of the same kind in
    /// the file, on the same side first, keeps its table, and so its
    /// icon, color and comments; one the file lacks gets a table of its
    /// own. A side the file leaves out is written out from the defaults
    /// first.
    ///
    /// # Errors
    ///
    /// Returns the [`Error::Config`](crate::Error::Config) loading the
    /// file would give. The file is left as it was.
    pub fn set_widgets(&mut self, widgets: &BarWidgets) -> Result<()> {
        if self.widgets()? == *widgets {
            return Ok(());
        }
        let mut pool = self.entries()?;
        let mut places = positions(&pool).into_iter();
        let mut written = Vec::new();
        for zone in BarZone::ALL {
            let tables: Vec<_> = widgets
                .zone(zone)
                .iter()
                .map(|w| {
                    let mut table = take(&mut pool, zone, w.kind);
                    set_enabled(&mut table, w.enabled);
                    table
                })
                .collect();
            written.push((zone, tables));
        }
        let mut last = None;
        for (zone, mut tables) in written {
            for table in &mut tables {
                last = places.next().or(last);
                if let Some(at) = last {
                    table.set_position(at);
                }
            }
            let inline = self.doc.get(zone.name()).is_some_and(Item::is_value);
            self.doc[zone.name()] = to_item(tables, inline);
        }
        self.dirty = true;
        Ok(())
    }

    /// The widget tables of both sides, left first, with the defaults
    /// standing in for a side the file leaves out.
    fn entries(&self) -> Result<Vec<(BarZone, Entry)>> {
        let defaults = BarConfig::default();
        let mut entries = Vec::new();
        for zone in BarZone::ALL {
            let tables = match self.doc.get(zone.name()) {
                Some(item) => tables_of(item),
                None => default_tables(zone, &defaults).map_err(|e| self.error(e))?,
            };
            for table in tables {
                let kind = table
                    .get("type")
                    .and_then(Item::as_str)
                    .and_then(|name| WidgetKind::from_str(name).ok());
                if let Some(kind) = kind {
                    entries.push((
                        zone,
                        Entry {
                            kind,
                            table,
                            used: false,
                        },
                    ));
                }
            }
        }
        Ok(entries)
    }
}

/// The places the tables have in the file, in order. Those that move
/// take the places of those that were there.
fn positions(pool: &[(BarZone, Entry)]) -> Vec<usize> {
    let mut places: Vec<_> = pool
        .iter()
        .filter_map(|(_, e)| e.table.position())
        .collect();
    places.sort_unstable();
    places
}

/// The table of the first unused widget of `kind`, on `zone` first.
fn take(pool: &mut [(BarZone, Entry)], zone: BarZone, kind: WidgetKind) -> Table {
    let index = [zone, zone.other()].into_iter().find_map(|side| {
        pool.iter()
            .position(|(z, e)| *z == side && e.kind == kind && !e.used)
    });
    match index.and_then(|i| pool.get_mut(i)) {
        Some((_, entry)) => {
            entry.used = true;
            entry.table.clone()
        }
        None => {
            let mut table = Table::new();
            table.insert("type", value(kind.name()));
            table
        }
    }
}

fn set_enabled(table: &mut Table, enabled: bool) {
    let now = table.get("enabled").and_then(Item::as_bool).unwrap_or(true);
    if now == enabled {
        return;
    }
    if enabled {
        table.remove("enabled");
        return;
    }
    let mut off = Value::from(false);
    if let Some(old) = table.get("enabled").and_then(Item::as_value) {
        *off.decor_mut() = old.decor().clone();
    }
    table.insert("enabled", Item::Value(off));
}

fn tables_of(item: &Item) -> Vec<Table> {
    match item {
        Item::ArrayOfTables(tables) => tables.iter().cloned().collect(),
        Item::Value(Value::Array(values)) => values
            .iter()
            .filter_map(Value::as_inline_table)
            .map(|t| t.clone().into_table())
            .collect(),
        _ => Vec::new(),
    }
}

fn default_tables(zone: BarZone, defaults: &BarConfig) -> std::result::Result<Vec<Table>, String> {
    let side = match zone {
        BarZone::Left => Side {
            left: Some(&defaults.left),
            right: None,
        },
        BarZone::Right => Side {
            left: None,
            right: Some(&defaults.right),
        },
    };
    let text = toml::to_string(&side).map_err(|e| e.to_string())?;
    let doc: DocumentMut = text
        .parse()
        .map_err(|e: toml_edit::TomlError| e.to_string())?;
    Ok(doc.get(zone.name()).map(tables_of).unwrap_or_default())
}

/// The tables as the item of a side. A side without widgets is an empty
/// array, as no tables at all would give it the defaults.
fn to_item(tables: Vec<Table>, inline: bool) -> Item {
    if tables.is_empty() || inline {
        let values = tables
            .into_iter()
            .map(|t| Value::InlineTable(t.into_inline_table()));
        return Item::Value(Value::Array(values.collect::<Array>()));
    }
    let mut list = ArrayOfTables::new();
    tables.into_iter().for_each(|t| list.push(t));
    Item::ArrayOfTables(list)
}
