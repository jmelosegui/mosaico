use std::path::Path;

use crate::Result;

use super::{append_to_file, file_error, read_file};

/// Appends `snippet`, a comment block about the `[section]` section, to
/// the config at `path`, unless the file already has the section, active
/// or commented out, as the template writes it and as an earlier run of
/// the hint does.
///
/// Returns `Ok(true)` when it appended the block. A second run finds the
/// block and does nothing.
pub(super) fn append_section_hint(path: &Path, section: &str, snippet: &str) -> Result<bool> {
    let content = read_file(path)?;

    let doc: toml_edit::DocumentMut = content.parse().map_err(|e| file_error(path, e))?;
    let has_header = content.lines().any(|line| is_header(line, section));
    if doc.contains_key(section) || has_header {
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

/// Whether `line` is the `[section]` header, commented out or not, with
/// spaces allowed inside the brackets. A line that only mentions the
/// section is not.
fn is_header(line: &str, section: &str) -> bool {
    let line = line.trim_start().trim_start_matches('#').trim_start();
    let key = line.strip_prefix('[').and_then(|rest| rest.split_once(']'));
    key.is_some_and(|(key, _)| key.trim() == section)
}
