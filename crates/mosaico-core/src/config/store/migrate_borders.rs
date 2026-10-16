use std::path::Path;

use crate::Result;

use super::file_error;

/// Moves top-level `focused` / `monocle` keys from `[borders]` into a
/// `[borders.colors]` sub-table.
///
/// If the user already has `[borders.colors].focused` or `.monocle`,
/// the existing nested values win and the legacy keys are dropped.
pub(super) fn migrate_border_colors(doc: &mut toml_edit::DocumentMut, path: &Path) -> Result<()> {
    let borders = doc.get_mut("borders").and_then(|t| t.as_table_mut());
    let Some(borders) = borders else {
        return Ok(());
    };

    let legacy_focused = borders.remove("focused");
    let legacy_monocle = borders.remove("monocle");
    if legacy_focused.is_none() && legacy_monocle.is_none() {
        return Ok(());
    }

    if !borders.contains_key("colors") {
        borders.insert("colors", toml_edit::Item::Table(toml_edit::Table::new()));
    }
    let colors = borders
        .get_mut("colors")
        .and_then(|t| t.as_table_mut())
        .ok_or_else(|| file_error(path, "[borders.colors] is not a table -- cannot migrate"))?;

    if let Some(item) = legacy_focused
        && !colors.contains_key("focused")
    {
        colors.insert("focused", item);
    }
    if let Some(item) = legacy_monocle
        && !colors.contains_key("monocle")
    {
        colors.insert("monocle", item);
    }
    Ok(())
}
