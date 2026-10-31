use std::path::Path;

use crate::Result;

use super::merge_section_hint::append_section_hint;

/// Comment block appended to `config.toml` when the file has no `[tray]`
/// section, so existing users find out how to hide the icon that appears
/// in the notification area. It is all comments: the file keeps the
/// defaults until the user writes the section.
pub(super) const TRAY_HINT_SNIPPET: &str = r##"
# Added automatically -- new in this version of mosaico
# The [tray] section puts the mosaico icon in the notification area while
# the daemon runs. Click it to open the settings; its menu also pauses the
# hotkeys, reloads the config and quits mosaico. Recognized keys:
#   enabled   whether the icon is shown: true or false.
#
# Example, to hide the icon:
#   [tray]
#   enabled = false
"##;

/// Appends a comment block about the notification area icon to the
/// config at `path`, unless the file already has a `[tray]` section,
/// active or commented out.
///
/// Returns `Ok(true)` when it appended the block. A second run finds the
/// block and does nothing.
pub(super) fn append_missing_tray_hint(path: &Path) -> Result<bool> {
    append_section_hint(path, "tray", TRAY_HINT_SNIPPET)
}

#[cfg(test)]
#[path = "tests/merge_tray_tests.rs"]
mod tests;
