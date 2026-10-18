use std::path::Path;

use crate::Result;

use super::{append_to_file, file_error, read_file};

/// Comment block appended to `config.toml` when the file never mentions
/// `drag_to_swap`, so existing users learn that dropping a window they
/// drag moves it. It is all comments: the file keeps the defaults until
/// the user writes the key. It names no default value, since it stays in
/// the file when the defaults change.
pub(super) const DRAG_TO_SWAP_HINT_SNIPPET: &str = r##"
# Added automatically -- new in this version of mosaico
# `drag_to_swap` under [mouse]: a tiled window you drag by its title bar
# and drop on another tile swaps places with it, and one you drop on
# another monitor goes before the tile under the pointer there. Set it to
# false to have a dropped window go back to its slot, or join the end of
# the other monitor's windows.
#
# Example:
#   [mouse]
#   drag_to_swap = false
"##;

/// Appends a comment block about `drag_to_swap` to the config at `path`,
/// unless the file mentions it already, as a key, a comment, the template
/// or an earlier run of this hint does.
///
/// Returns `Ok(true)` when it appended the block. A second run finds the
/// block and does nothing.
pub(super) fn append_missing_drag_to_swap_hint(path: &Path) -> Result<bool> {
    let content = read_file(path)?;

    let _: toml_edit::DocumentMut = content.parse().map_err(|e| file_error(path, e))?;
    if content.contains("drag_to_swap") {
        return Ok(false);
    }

    let mut to_append = String::new();
    if !content.ends_with('\n') {
        to_append.push('\n');
    }
    to_append.push_str(DRAG_TO_SWAP_HINT_SNIPPET);

    append_to_file(path, &to_append)?;
    Ok(true)
}

#[cfg(test)]
#[path = "tests/merge_mouse_tests.rs"]
mod tests;
