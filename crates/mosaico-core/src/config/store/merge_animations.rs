use std::path::Path;

use crate::Result;

use super::merge_section_hint::append_section_hint;

/// Comment block appended to `config.toml` when the file has no
/// `[animations]` section, so existing users discover window transitions
/// without having to read release notes. It is all comments: the file
/// keeps the defaults until the user writes the section. It names no
/// default value, since it stays in the file when the defaults change.
pub(super) const ANIMATIONS_HINT_SNIPPET: &str = r##"
# Added automatically -- new in this version of mosaico
# The [animations] section moves windows to their new place over a short
# time when they swap, move or resize, and when the layout changes,
# instead of jumping there. Recognized keys:
#   enabled        whether windows move this way: true or false.
#   duration       how long a transition takes, in milliseconds, 0 to
#                  1000.
#   easing         "linear", "ease-out-quad", "ease-out-cubic",
#                  "ease-in-out-cubic", "ease-out-expo", or
#                  "cubic-bezier(x1, y1, x2, y2)" with all four numbers
#                  from 0 to 1.
#   follow_system  true to snap windows into place while animations are
#                  turned off in the system's settings, and in a remote
#                  session.
#
# Example, to have windows jump into place:
#   [animations]
#   enabled = false
"##;

/// Appends a comment block about window transitions to the config at
/// `path`, unless the file already has an `[animations]` section, active
/// or commented out, as the template writes it and as an earlier run of
/// this hint does.
///
/// Returns `Ok(true)` when it appended the block. A second run finds the
/// block and does nothing.
pub(super) fn append_missing_animations_hint(path: &Path) -> Result<bool> {
    append_section_hint(path, "animations", ANIMATIONS_HINT_SNIPPET)
}
#[cfg(test)]
#[path = "tests/merge_animations_tests.rs"]
mod tests;
