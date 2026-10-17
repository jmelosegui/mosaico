use std::path::Path;

use crate::Result;

use super::merge_animations::append_missing_animations_hint;
use super::merge_unfocused::append_missing_unfocused_border_hint;
use super::merge_wrap::add_missing_wrap_key;
use super::{ConfigStore, append_to_file, file_error, read_file};

/// Snippet appended to `config.toml` when the `[workspaces]` section is
/// missing. Documents the new options so users notice them on first load
/// after upgrading.
const WORKSPACES_SECTION_SNIPPET: &str = r##"
# Added automatically -- new defaults from this version of mosaico
[workspaces]
# How a workspace switch is applied across monitors.
# "per-monitor" (default): only the focused monitor switches.
# "global": all monitors switch in lockstep, like Windows virtual desktops.
mode = "per-monitor"
# Whether stepping past the last workspace, with goto-workspace-next or the
# wheel over the bar, goes back to the first, and before the first to the
# last. false stops at either end.
wrap = false

# Per-workspace layout overrides (workspace number 1-8).
# Available layouts: "bsp", "vertical-stack", "three-column".
# [workspaces.layouts]
# 1 = "vertical-stack"
# 3 = "three-column"
"##;

impl ConfigStore {
    /// Adds what newer versions of mosaico introduced to `config.toml`:
    ///
    /// - A documented `[workspaces]` section, appended when the file has
    ///   none. One the file has from before `wrap` gets the key, the way
    ///   the `unfocused` color is added below (see `merge_wrap.rs`).
    /// - The `unfocused` border color. When `[borders.colors]` is an
    ///   active table without the key, `unfocused = ""` is inserted into
    ///   it under a comment, which rewrites the file in place without a
    ///   backup. Otherwise a comment block about it is appended, unless
    ///   the file already mentions `unfocused` anywhere.
    /// - Window transitions. A comment block about the `[animations]`
    ///   section is appended, unless the file has the section, active or
    ///   commented out.
    ///
    /// Mirrors [`merge_missing_keybindings`](Self::merge_missing_keybindings)
    /// and [`merge_missing_bar_widgets`](Self::merge_missing_bar_widgets):
    /// what future versions of mosaico introduce is added as commented or
    /// default-valued entries so users discover it without losing their
    /// existing customizations.
    ///
    /// No-op when the file does not exist (a brand-new install will get the
    /// full section from `mosaico init`'s template). Errors are logged and
    /// swallowed -- failure here must not prevent the daemon from starting.
    /// Only the daemon calls this, so it reports to the log alone.
    pub fn merge_missing_config_sections(&self) {
        let path = self.config_path();
        if !path.exists() {
            return;
        }

        match append_missing_workspaces_section(&path) {
            Ok(true) => crate::log_info!(
                "Appended [workspaces] section to {} (new in this version)",
                path.display()
            ),
            Ok(false) => {}
            Err(e) => crate::log_warn!("Could not add the [workspaces] section: {e}"),
        }

        match add_missing_wrap_key(&path) {
            Ok(true) => crate::log_info!(
                "Added [workspaces] wrap to {} (new in this version)",
                path.display()
            ),
            Ok(false) => {}
            Err(e) => crate::log_warn!("Could not add [workspaces] wrap: {e}"),
        }

        match append_missing_unfocused_border_hint(&path) {
            Ok(true) => crate::log_info!(
                "Appended unfocused border hint to {} (new in this version)",
                path.display()
            ),
            Ok(false) => {}
            Err(e) => crate::log_warn!("Could not add the unfocused border hint: {e}"),
        }

        match append_missing_animations_hint(&path) {
            Ok(true) => crate::log_info!(
                "Appended [animations] hint to {} (new in this version)",
                path.display()
            ),
            Ok(false) => {}
            Err(e) => crate::log_warn!("Could not add the [animations] hint: {e}"),
        }
    }
}

/// Path-pure helper for [`ConfigStore::merge_missing_config_sections`]: returns
/// `Ok(true)` when an append happened, `Ok(false)` when the file already
/// has a `[workspaces]` section.
fn append_missing_workspaces_section(path: &Path) -> Result<bool> {
    let content = read_file(path)?;

    let doc: toml_edit::DocumentMut = content.parse().map_err(|e| file_error(path, e))?;

    if doc.contains_key("workspaces") {
        return Ok(false);
    }

    // Make sure the snippet starts on a fresh line even if the existing
    // file does not end in a newline.
    let mut to_append = String::new();
    if !content.ends_with('\n') {
        to_append.push('\n');
    }
    to_append.push_str(WORKSPACES_SECTION_SNIPPET);

    append_to_file(path, &to_append)?;
    Ok(true)
}

#[cfg(test)]
#[path = "tests/merge_config_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/merge_sections_tests.rs"]
mod sections_tests;
