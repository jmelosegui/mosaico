use std::path::Path;

use crate::Result;

use super::write_back::write_back;
use super::{append_to_file, file_error, read_file};

/// Comment block appended to `config.toml` when its `workspaces` section
/// is not a table the `wrap` key can go in, such as an inline table, and
/// the file does not mention `wrap`, so existing users find out about it.
pub(super) const WRAP_HINT_SNIPPET: &str = r##"
# Added automatically -- new in this version of mosaico
# The `wrap` key under [workspaces] says whether stepping past the last
# workspace, with goto-workspace-next or the wheel over the bar, goes back
# to the first, and before the first to the last. Recognized values:
#   false   stop at either end. This is the default when the key is absent.
#   true    go round.
#
# Example:
#   [workspaces]
#   wrap = true
"##;

/// Comment placed above the `wrap` key inserted into an active
/// `[workspaces]` table, worded as the section the template writes.
pub(super) const WRAP_INLINE_PREFIX: &str = "\n\
    # Whether stepping past the last workspace, with goto-workspace-next or the\n\
    # wheel over the bar, goes back to the first, and before the first to the\n\
    # last. false stops at either end.\n";

/// Adds a documented `wrap` key to a config whose `[workspaces]` section
/// is from before it, the way the `unfocused` border color is added:
///
/// 1. When `[workspaces]` is an active table of its own without the key,
///    `wrap = false`, the default, is inserted into it under a comment,
///    which rewrites the file in place without a backup, in its line
///    endings and with its byte order mark (see `write_back.rs`).
/// 2. Otherwise, when the file has the section in another form and does
///    not mention `wrap` anywhere, a comment block about it is appended.
///
/// A file without the section is left alone: the section appended to it
/// has the key. Returns `Ok(true)` when the file changed. A second run
/// finds the key, or the mention, and does nothing.
pub(super) fn add_missing_wrap_key(path: &Path) -> Result<bool> {
    let content = read_file(path)?;
    let mut doc: toml_edit::DocumentMut = content.parse().map_err(|e| file_error(path, e))?;
    let Some(section) = doc.get_mut("workspaces") else {
        return Ok(false);
    };

    let own_table = section
        .as_table_mut()
        .filter(|table| !table.is_implicit() && !table.is_dotted());
    if let Some(table) = own_table {
        if table.contains_key("wrap") {
            return Ok(false);
        }
        table.insert("wrap", toml_edit::value(false));
        if let Some(mut key) = table.key_mut("wrap") {
            key.leaf_decor_mut()
                .set_prefix(toml_edit::RawString::from(WRAP_INLINE_PREFIX));
        }
        write_back(path, &content, &doc.to_string())
            .map_err(|e| file_error(path, format_args!("could not write it: {e}")))?;
        return Ok(true);
    }

    if content.contains("wrap") {
        return Ok(false);
    }
    let mut to_append = String::new();
    if !content.ends_with('\n') {
        to_append.push('\n');
    }
    to_append.push_str(WRAP_HINT_SNIPPET);
    append_to_file(path, &to_append)?;
    Ok(true)
}

#[cfg(test)]
#[path = "tests/merge_wrap_tests.rs"]
mod tests;
