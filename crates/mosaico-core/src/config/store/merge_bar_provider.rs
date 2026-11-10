use std::path::Path;

use crate::Result;

use super::merge_section_hint::append_section_hint;

/// Comment block appended to `config.toml` when the file has no `[bar]`
/// section, so existing users find out they can hand the bar to another
/// program or have none. It is all comments: the file keeps mosaico's own
/// bar until the user writes the section.
pub(super) const BAR_PROVIDER_HINT_SNIPPET: &str = r##"
# Added automatically -- new in this version of mosaico
# The [bar] section picks the bar that shows your workspaces. Tiles keep
# clear of it. Recognized keys:
#   provider   "mosaico" (default): mosaico's own bar, styled in bar.toml.
#              "yasb": YASB; `mosaico bar setup yasb` prints its widgets.
#              "external": any other program's bar. For both, bar.toml is
#              not read and tiles keep clear of the space the bar asks for.
#              "none": no bar, and no space kept for one.
#   [bar.reserve] top, bottom, left, right: space kept for a "yasb" or
#              "external" bar that cannot ask, in pixels at 100% scale.
#
# Example, to use another program's bar:
#   [bar]
#   provider = "external"
#   [bar.reserve]
#   top = 32
"##;

/// Appends a comment block about picking the bar to the config at `path`,
/// unless the file already has a `[bar]` section, active or commented out.
///
/// Returns `Ok(true)` when it appended the block. A second run finds the
/// block and does nothing.
pub(super) fn append_missing_bar_provider_hint(path: &Path) -> Result<bool> {
    append_section_hint(path, "bar", BAR_PROVIDER_HINT_SNIPPET)
}

#[cfg(test)]
#[path = "tests/merge_bar_provider_tests.rs"]
mod tests;
