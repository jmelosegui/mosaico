//! What a config file changed through `toml_edit` is written back as.

use super::*;

#[test]
fn a_file_with_lf_endings_keeps_them() {
    // Arrange
    let original = "[workspaces]\nmode = \"global\"\n";

    // Act
    let text = as_written(original, "[workspaces]\nmode = \"global\"\nwrap = false\n");

    // Assert
    assert_eq!(text, "[workspaces]\nmode = \"global\"\nwrap = false\n");
}

#[test]
fn a_file_with_crlf_endings_gets_them_on_every_line() {
    // Arrange -- including the lines the edit added.
    let original = "[workspaces]\r\nmode = \"global\"\r\n";

    // Act
    let text = as_written(original, "[workspaces]\nmode = \"global\"\nwrap = false\n");

    // Assert
    assert_eq!(
        text,
        "[workspaces]\r\nmode = \"global\"\r\nwrap = false\r\n"
    );
}

#[test]
fn a_file_that_starts_with_a_byte_order_mark_keeps_it_once() {
    // Arrange
    let original = "\u{feff}[workspaces]\r\nmode = \"global\"\r\n";

    // Act -- whether or not the edit dropped it.
    let dropped = as_written(original, "[workspaces]\nwrap = false\n");
    let kept = as_written(original, "\u{feff}[workspaces]\nwrap = false\n");

    // Assert
    let expected = "\u{feff}[workspaces]\r\nwrap = false\r\n";
    assert_eq!((dropped.as_str(), kept.as_str()), (expected, expected));
}

#[test]
fn a_file_without_a_byte_order_mark_gets_none() {
    // Arrange
    let original = "wrap = true";

    // Act
    let text = as_written(original, "\u{feff}wrap = true");

    // Assert
    assert_eq!(text, "wrap = true");
}

#[test]
fn a_file_of_one_line_takes_lf_endings() {
    // Arrange -- nothing says how its lines end.
    let original = "[workspaces]";

    // Act
    let text = as_written(original, "[workspaces]\nwrap = false\n");

    // Assert
    assert_eq!(text, "[workspaces]\nwrap = false\n");
}
