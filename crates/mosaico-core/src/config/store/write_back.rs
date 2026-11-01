//! Writing a config file back once `toml_edit` has changed it, the way
//! the file was written: `toml_edit` ends every line with LF and drops a
//! byte order mark, either of which an editor may have written.

use std::path::Path;

/// The byte order mark of UTF-8, as text.
const BOM: char = '\u{feff}';

/// Writes `edited`, what `toml_edit` made of `original`, the text the
/// config file at `path` held, back to the file in the line endings of
/// `original` and with the byte order mark it starts with, if it does.
/// Creates the file's directory when it does not exist, as on an install
/// that has no config files yet. Returns the text written.
pub(super) fn write_back(path: &Path, original: &str, edited: &str) -> std::io::Result<String> {
    let text = as_written(original, edited);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(path, &text)?;
    Ok(text)
}

/// `edited` in the line endings of `original`, CRLF where the first line
/// of `original` ends with CRLF and LF otherwise, and with the byte order
/// mark `original` starts with, if it does.
pub(super) fn as_written(original: &str, edited: &str) -> String {
    let lines = edited
        .strip_prefix(BOM)
        .unwrap_or(edited)
        .replace("\r\n", "\n");
    let text = if ends_lines_with_crlf(original) {
        lines.replace('\n', "\r\n")
    } else {
        lines
    };
    if original.starts_with(BOM) {
        format!("{BOM}{text}")
    } else {
        text
    }
}

/// Whether the first line of `text` ends with CRLF.
fn ends_lines_with_crlf(text: &str) -> bool {
    text.find('\n')
        .is_some_and(|end| text[..end].ends_with('\r'))
}

#[cfg(test)]
#[path = "tests/write_back_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/write_back_merge_tests.rs"]
mod merge_tests;
