use std::path::Path;

use crate::Result;

use super::{append_to_file, file_error, read_file};

/// Comment block appended to `bar.toml` when the file has no `[mouse]`
/// section, so existing users find out that the bar takes clicks and the
/// wheel now, and how to turn that off. It is all comments: the file
/// keeps the defaults until the user writes the section. It names no
/// default value, since it stays in the file when the defaults change.
pub(super) const BAR_MOUSE_HINT_SNIPPET: &str = r##"
# Added automatically -- new in this version of mosaico
# The [mouse] section says what the mouse does on the bar: a click on a
# workspace number goes to that workspace, and the wheel steps through the
# workspaces, on the bar's own monitor. Recognized keys:
#   enabled  whether the bar takes clicks and the wheel: true or false.
#            With false the bar lets the pointer through to the windows
#            under it.
#   scroll   what scrolling over the bar does: "workspaces" steps through
#            the workspaces of the bar's monitor, "none" does nothing.
#
# Example, to keep the bar out of the pointer's way:
#   [mouse]
#   enabled = false
"##;

/// Appends a comment block about the bar's `[mouse]` section to the bar
/// config at `path`, unless the file already has the section, active or
/// commented out, as the template writes it and as an earlier run of this
/// hint does.
///
/// Returns `Ok(true)` when it appended the block. A second run finds the
/// block and does nothing.
pub(super) fn append_missing_bar_mouse_hint(path: &Path) -> Result<bool> {
    let content = read_file(path)?;

    let doc: toml_edit::DocumentMut = content.parse().map_err(|e| file_error(path, e))?;
    if doc.contains_key("mouse") || content.lines().any(is_mouse_header) {
        return Ok(false);
    }

    let mut to_append = String::new();
    if !content.ends_with('\n') {
        to_append.push('\n');
    }
    to_append.push_str(BAR_MOUSE_HINT_SNIPPET);

    append_to_file(path, &to_append)?;
    Ok(true)
}

/// Whether `line` is the `[mouse]` header, commented out or not, with
/// spaces allowed inside the brackets. A line that only mentions the
/// section is not, and neither is the header of an array of tables in it.
fn is_mouse_header(line: &str) -> bool {
    let line = line.trim_start().trim_start_matches('#').trim_start();
    let key = line.strip_prefix('[').and_then(|rest| rest.split_once(']'));
    key.is_some_and(|(key, _)| key.trim() == "mouse")
}

#[cfg(test)]
#[path = "tests/merge_bar_mouse_tests.rs"]
mod tests;
