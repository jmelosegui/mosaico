//! What `mosaico doctor` says about the values written in `config.toml`
//! and `bar.toml`, in the words the settings window uses for them.

use std::path::Path;

use mosaico_core::config::schema::{SettingDescriptor, review};

use super::context::{Context, say};

const WARN: &str = "\x1b[33m[warn]\x1b[0m";

/// Says each value of the file `name` at `path` that mosaico does not
/// take as written.
pub fn check_settings(
    cx: &mut Context,
    name: &str,
    path: &Path,
    settings: &'static [SettingDescriptor],
) {
    for line in settings_lines(name, path, settings) {
        say!(cx, "{line}");
    }
}

/// The lines [`check_settings`] says. A file that is missing has the
/// defaults, and one that does not parse is reported by the file check,
/// so neither has any.
fn settings_lines(name: &str, path: &Path, settings: &'static [SettingDescriptor]) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    review(&text, settings)
        .iter()
        .map(|problem| format!("  {WARN} {name}: {problem}"))
        .collect()
}

#[cfg(test)]
#[path = "tests/doctor_settings_tests.rs"]
mod tests;
