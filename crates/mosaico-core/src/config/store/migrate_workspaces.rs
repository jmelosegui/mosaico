use std::path::Path;

use crate::Result;

use super::file_error;

/// Moves `[layout.workspaces]` to `[workspaces.layouts]`.
///
/// If the user already has `[workspaces.layouts]`, keeps it and drops
/// the legacy mapping (the new section wins).
pub(super) fn migrate_workspaces_section(
    doc: &mut toml_edit::DocumentMut,
    path: &Path,
) -> Result<()> {
    let legacy = doc
        .get_mut("layout")
        .and_then(|t| t.as_table_mut())
        .and_then(|t| t.remove("workspaces"));

    let Some(legacy_item) = legacy else {
        return Ok(());
    };

    if doc.get("workspaces").is_none() {
        doc.insert(
            "workspaces",
            toml_edit::Item::Table(toml_edit::Table::new()),
        );
    }
    let workspaces = doc
        .get_mut("workspaces")
        .and_then(|t| t.as_table_mut())
        .ok_or_else(|| file_error(path, "[workspaces] is not a table -- cannot migrate"))?;

    if !workspaces.contains_key("layouts") {
        workspaces.insert("layouts", legacy_item);
    }
    Ok(())
}
