use std::path::{Path, PathBuf};

use crate::Result;

use super::file_error;
use super::migrate_borders::migrate_border_colors;
use super::migrate_workspaces::migrate_workspaces_section;
use super::report;

/// Detects and applies all legacy -> current schema migrations:
///
/// - `[layout.workspaces]` -> `[workspaces.layouts]`
/// - `[borders] focused = ...` / `monocle = ...` -> `[borders.colors]`
///
/// On detection of any legacy pattern: backs up the original file
/// (aborting if the backup write fails), rewrites the config in place
/// using `toml_edit` so comments and formatting survive, and returns
/// the new content for the caller to parse.
///
/// Returns `Ok(None)` when no migration was needed.
pub(super) fn migrate_config_if_needed(path: &Path, content: &str) -> Result<Option<String>> {
    let mut doc: toml_edit::DocumentMut = content.parse().map_err(|e| file_error(path, e))?;

    let has_legacy_workspaces = doc
        .get("layout")
        .and_then(|t| t.as_table())
        .is_some_and(|t| t.contains_key("workspaces"));

    let has_legacy_borders = doc
        .get("borders")
        .and_then(|t| t.as_table())
        .is_some_and(|t| t.contains_key("focused") || t.contains_key("monocle"));

    if !has_legacy_workspaces && !has_legacy_borders {
        return Ok(None);
    }

    // Back up first -- abort migration if backup fails so we never mutate
    // without a safety net.
    let backup = backup_path(path);
    std::fs::copy(path, &backup).map_err(|e| {
        file_error(
            path,
            format_args!("could not back up to {}: {e}", backup.display()),
        )
    })?;

    let mut applied: Vec<&'static str> = Vec::new();

    if has_legacy_workspaces {
        migrate_workspaces_section(&mut doc, path)?;
        applied.push("[layout.workspaces] -> [workspaces.layouts]");
    }

    if has_legacy_borders {
        migrate_border_colors(&mut doc, path)?;
        applied.push("[borders] focused/monocle -> [borders.colors]");
    }

    let new_content = doc.to_string();
    std::fs::write(path, &new_content).map_err(|e| {
        file_error(
            path,
            format_args!("could not write the migrated config: {e}"),
        )
    })?;

    report::info(format_args!(
        "migrated config keys ({}). Backup saved at {}",
        applied.join(", "),
        backup.display()
    ));

    Ok(Some(new_content))
}

/// Returns a non-clobbering backup path next to `path`.
///
/// Uses `<stem>.pre-0.9.bak` as the base name so the user can tell what
/// the backup is for. Appends a numeric suffix if it already exists.
fn backup_path(path: &Path) -> PathBuf {
    let base = path.with_extension("toml.pre-0.9.bak");
    if !base.exists() {
        return base;
    }
    for n in 1..1000 {
        let candidate = path.with_extension(format!("toml.pre-0.9.bak.{n}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    base
}

#[cfg(test)]
#[path = "tests/migrate_tests.rs"]
mod tests;
