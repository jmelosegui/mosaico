use std::path::Path;

use crate::Result;

use super::{append_to_file, file_error, read_file};

/// Comment block appended to `config.toml` when the file has no
/// reference to the `unfocused` border color, so existing users
/// discover the new option without having to read release notes.
pub(super) const UNFOCUSED_BORDER_HINT_SNIPPET: &str = r##"
# Added automatically -- new in this version of mosaico
# The `unfocused` color under [borders.colors] sets the border drawn
# around tiled windows that do not currently have focus. Recognized
# values:
#   ""           use the active theme's muted gray default (Mocha:
#                #6c7086). This is the default when the field is
#                absent.
#   "none"       disable unfocused borders entirely; only the focused
#                window will get a border.
#   "#RRGGBB"    explicit hex color.
#   "blue"       any named theme color (mauve, teal, lavender, etc.).
#
# Example:
#   [borders.colors]
#   unfocused = "none"
"##;

/// Adds a documented `unfocused` field to the user's config so the
/// new option is discoverable.
///
/// Two strategies, in order:
/// 1. If `[borders.colors]` is an active table, insert `unfocused = ""`
///    into it with a leading comment. Empty string preserves current
///    behavior (theme-default muted gray).
/// 2. Otherwise (no active table, or the table is commented out), if
///    the file does not already mention `unfocused`, append a
///    standalone comment block at the end of the file.
///
/// Both paths are idempotent: a second run finds the key (or the
/// substring) already present and does nothing.
pub(super) fn append_missing_unfocused_border_hint(path: &Path) -> Result<bool> {
    let content = read_file(path)?;

    let inserted =
        insert_unfocused_into_active_section(&content).map_err(|e| file_error(path, e))?;
    if let Some(updated) = inserted {
        std::fs::write(path, updated)
            .map_err(|e| file_error(path, format_args!("could not write it: {e}")))?;
        return Ok(true);
    }

    // Fall back to the comment-only hint when there is no active
    // [borders.colors] section to extend. Skip when any reference to
    // `unfocused` already exists (active key, comment, prior hint).
    if content.contains("unfocused") {
        return Ok(false);
    }

    let mut to_append = String::new();
    if !content.ends_with('\n') {
        to_append.push('\n');
    }
    to_append.push_str(UNFOCUSED_BORDER_HINT_SNIPPET);

    append_to_file(path, &to_append)?;
    Ok(true)
}

/// Inline comment placed above the inserted `unfocused` key. Kept on
/// its own constant so the tests can pin the wording without growing
/// the inline path.
const UNFOCUSED_INLINE_PREFIX: &str = "\n\
    # Color drawn around unfocused tiled windows.\n\
    # \"none\" disables unfocused borders; \"\" (empty) uses the\n\
    # theme default (muted gray); a hex code or named theme color\n\
    # works too.\n";

/// If `content` has an active `[borders.colors]` table that is missing
/// the `unfocused` key, returns the rewritten file contents with the
/// key inserted. Returns `Ok(None)` when no edit is needed.
fn insert_unfocused_into_active_section(
    content: &str,
) -> Result<Option<String>, toml_edit::TomlError> {
    let mut doc: toml_edit::DocumentMut = content.parse()?;

    let Some(colors) = doc
        .get_mut("borders")
        .and_then(|t| t.as_table_mut())
        .and_then(|t| t.get_mut("colors"))
        .and_then(|t| t.as_table_mut())
    else {
        return Ok(None);
    };

    if colors.contains_key("unfocused") {
        return Ok(None);
    }

    colors.insert("unfocused", toml_edit::value(""));
    if let Some(mut key) = colors.key_mut("unfocused") {
        key.leaf_decor_mut()
            .set_prefix(toml_edit::RawString::from(UNFOCUSED_INLINE_PREFIX));
    }

    Ok(Some(doc.to_string()))
}

#[cfg(test)]
#[path = "tests/merge_unfocused_tests.rs"]
mod tests;
