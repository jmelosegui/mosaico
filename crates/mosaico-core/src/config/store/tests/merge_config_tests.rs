use crate::config::store::test_support::{cleanup, unique_temp_path};

use super::*;

#[test]
fn appends_workspaces_section_when_missing() {
    let path = unique_temp_path();
    let original = "\
[layout]
gap = 8

[borders]
width = 4
";
    std::fs::write(&path, original).unwrap();

    let appended = append_missing_workspaces_section(&path).unwrap();
    assert!(appended, "should append when [workspaces] is missing");

    let on_disk = std::fs::read_to_string(&path).unwrap();
    assert!(
        on_disk.starts_with(original),
        "existing content must be preserved verbatim"
    );
    assert!(on_disk.contains("[workspaces]"));
    assert!(on_disk.contains("mode = \"per-monitor\""));
    assert!(on_disk.contains("# [workspaces.layouts]"));

    cleanup(&path);
}

#[test]
fn does_not_append_when_section_present() {
    let path = unique_temp_path();
    let original = "\
[layout]
gap = 8

[workspaces]
mode = \"global\"
";
    std::fs::write(&path, original).unwrap();

    let appended = append_missing_workspaces_section(&path).unwrap();
    assert!(!appended, "should not append when section already present");

    let on_disk = std::fs::read_to_string(&path).unwrap();
    assert_eq!(on_disk, original, "file must be untouched");

    cleanup(&path);
}

#[test]
fn append_handles_file_without_trailing_newline() {
    let path = unique_temp_path();
    let original = "[layout]\ngap = 8"; // no trailing newline
    std::fs::write(&path, original).unwrap();

    append_missing_workspaces_section(&path).unwrap();

    let on_disk = std::fs::read_to_string(&path).unwrap();
    // The section header must land on its own line, not be glued to
    // `gap = 8`.
    assert!(on_disk.contains("gap = 8\n"));
    assert!(on_disk.contains("[workspaces]"));

    cleanup(&path);
}
