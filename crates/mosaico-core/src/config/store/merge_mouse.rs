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

/// Comment block appended to `config.toml` when the file never mentions
/// `resize_splits`, so existing users learn that dragging the edge between
/// two tiles resizes them. Like the `drag_to_swap` block, it is all
/// comments and names no default value.
pub(super) const RESIZE_SPLITS_HINT_SNIPPET: &str = r##"
# Added automatically -- new in this version of mosaico
# `resize_splits` under [mouse]: drag the edge between two tiles to resize
# them, and the split between them follows the edge; a corner where two
# splits meet moves both. Set it to false to have a tiled window you
# resize go back to its slot.
#
# Example:
#   [mouse]
#   resize_splits = false
"##;

/// Appends a comment block about `drag_to_swap` to the config at `path`,
/// unless the file mentions it already, as a key, a comment, the template
/// or an earlier run of this hint does.
///
/// Returns `Ok(true)` when it appended the block. A second run finds the
/// block and does nothing.
pub(super) fn append_missing_drag_to_swap_hint(path: &Path) -> Result<bool> {
    append_missing_hint(path, "drag_to_swap", DRAG_TO_SWAP_HINT_SNIPPET)
}

/// Appends a comment block about `resize_splits` to the config at `path`,
/// unless the file mentions it already, as
/// [`append_missing_drag_to_swap_hint`] does for its key.
pub(super) fn append_missing_resize_splits_hint(path: &Path) -> Result<bool> {
    append_missing_hint(path, "resize_splits", RESIZE_SPLITS_HINT_SNIPPET)
}

/// Appends the blocks about `drag_to_swap` and `resize_splits` to the
/// config at `path`, in the template's order, each one the file lacks, and
/// logs what it did. An error is logged, and the next block still tried,
/// as with every addition to the config.
pub(super) fn append_missing_mouse_hints(path: &Path) {
    let appended = [
        ("drag_to_swap", append_missing_drag_to_swap_hint(path)),
        ("resize_splits", append_missing_resize_splits_hint(path)),
    ];
    for (key, result) in appended {
        match result {
            Ok(true) => crate::log_info!(
                "Appended {key} hint to {} (new in this version)",
                path.display()
            ),
            Ok(false) => {}
            Err(e) => crate::log_warn!("Could not add the {key} hint: {e}"),
        }
    }
}

/// Appends `snippet` to the config at `path`, on a line of its own,
/// unless the file mentions `key` anywhere. A file that is not valid TOML
/// is left as it is, and the error names it.
fn append_missing_hint(path: &Path, key: &str, snippet: &str) -> Result<bool> {
    let content = read_file(path)?;

    let _: toml_edit::DocumentMut = content.parse().map_err(|e| file_error(path, e))?;
    if content.contains(key) {
        return Ok(false);
    }

    let mut to_append = String::new();
    if !content.ends_with('\n') {
        to_append.push('\n');
    }
    to_append.push_str(snippet);

    append_to_file(path, &to_append)?;
    Ok(true)
}

#[cfg(test)]
#[path = "tests/merge_mouse_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/merge_resize_tests.rs"]
mod resize_tests;
