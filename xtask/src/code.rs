//! Code health counts for each crate, read from its source files.
//!
//! The counts follow the rules in AGENTS.md and the rust-conventions
//! skill: files stay under 300 lines, `unsafe` stays small,
//! library code does not unwrap, and the portable core names no OS
//! concept.

use mosaico_metrics::{Better, Report};

use crate::scan::{braces, is_test_file, occurrences, words};
use crate::source::SourceFile;

/// How many lines a source file may have before it counts as too long.
pub const LINE_LIMIT: usize = 300;

/// Crates whose non-test code must not `.unwrap()` or `.expect()`.
const LIBRARY_CRATES: &[&str] = &["mosaico-core", "mosaico-windows"];

/// Crates that must stay portable.
const PORTABLE_CRATES: &[&str] = &["mosaico-core"];

/// OS names the portable crates must not mention, in code or docs.
const OS_NAMES: &[&str] = &[
    "HWND",
    "HMONITOR",
    "DWM",
    "Dwm",
    "ShowWindow",
    "EVENT_",
    "windows::",
    "target_os",
];

/// The counts of one source file.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FileCounts {
    /// `#[test]` functions.
    pub tests: u64,
    /// Lines beyond [`LINE_LIMIT`].
    pub lines_over_limit: u64,
    /// `unsafe` keywords in code, outside comments.
    pub unsafe_keywords: u64,
    /// `.unwrap()` and `.expect(` calls outside test code.
    pub unwraps: u64,
    /// Mentions of [`OS_NAMES`], comments included.
    pub os_names: u64,
}

/// Counts `text`, the contents of the file at `path` within its crate.
pub fn count_file(path: &str, text: &str) -> FileCounts {
    let lines = text.lines().count();
    let mut counts = FileCounts {
        lines_over_limit: lines.saturating_sub(LINE_LIMIT) as u64,
        os_names: OS_NAMES.iter().map(|n| occurrences(text, n)).sum(),
        ..FileCounts::default()
    };
    // An inline test module makes the rest of the file test code. Any
    // other item under `#[cfg(test)]` is test code up to its closing brace.
    let mut in_tests = is_test_file(path);
    let mut cfg_test = false;
    let mut test_item_depth = 0;
    for line in text.lines() {
        let trimmed = line.trim();
        let code = line.split("//").next().unwrap_or_default();
        let mut test_code = in_tests || test_item_depth > 0;
        if trimmed.starts_with("#[cfg(test)]") {
            cfg_test = true;
        } else if cfg_test && !trimmed.starts_with("#[") {
            cfg_test = false;
            test_code = true;
            if trimmed.contains("mod ") && trimmed.ends_with('{') {
                in_tests = true;
            } else {
                test_item_depth = braces(code);
            }
        } else if test_item_depth > 0 {
            test_item_depth += braces(code);
        }
        if trimmed.starts_with("#[test]") {
            counts.tests += 1;
        }
        counts.unsafe_keywords += words(code, "unsafe");
        if !test_code {
            counts.unwraps +=
                (code.matches(".unwrap()").count() + code.matches(".expect(").count()) as u64;
        }
    }
    counts
}

/// The counts of every crate, one section per crate.
pub fn report(files: &[SourceFile]) -> Report {
    let mut report = Report::new();
    for file in files {
        let counts = count_file(&file.path, &file.text);
        let name = file.crate_name.as_str();
        let mut add = |metric: &str, value: u64, better: Better| {
            let total = report.get(name, metric).map_or(0, |m| m.value);
            report.set(name, metric, total + value, better);
        };
        add("tests", counts.tests, Better::Higher);
        add(
            "files_over_line_limit",
            u64::from(counts.lines_over_limit > 0),
            Better::Lower,
        );
        add(
            "lines_over_line_limit",
            counts.lines_over_limit,
            Better::Lower,
        );
        add("unsafe_keywords", counts.unsafe_keywords, Better::Lower);
        if LIBRARY_CRATES.contains(&name) {
            add("unwraps_outside_tests", counts.unwraps, Better::Lower);
        }
        if PORTABLE_CRATES.contains(&name) {
            add("os_names", counts.os_names, Better::Lower);
        }
    }
    report
}

#[cfg(test)]
#[path = "code_tests.rs"]
mod tests;
