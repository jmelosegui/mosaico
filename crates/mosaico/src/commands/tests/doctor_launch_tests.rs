use std::path::Path;

use mosaico_core::config::{keybinding, parse_launches};

use super::*;

fn entries(content: &str) -> Result<Vec<Result<LaunchBinding>>> {
    parse_launches(Path::new("keybindings.toml"), content)
}

fn launch(command: &str, key: &str) -> String {
    format!("[[launch]]\ncommand = \"{command}\"\nkey = \"{key}\"\nmodifiers = [\"alt\"]\n")
}

#[test]
fn good_launch_bindings_are_counted_and_none_say_nothing() {
    // Arrange
    let keybindings = keybinding::defaults();
    let content = launch("wt.exe", "Enter") + &launch("msedge.exe", "B");

    // Act
    let some = launch_lines(&keybindings, entries(&content));
    let none = launch_lines(&keybindings, entries(""));

    // Assert
    assert_eq!(some.len(), 1, "{some:?}");
    assert!(
        some[0].contains("All 2 launch binding(s) are valid"),
        "{some:?}"
    );
    assert!(none.is_empty(), "{none:?}");
}

#[test]
fn bad_entries_and_unknown_keys_fail_one_by_one() {
    // Arrange
    let content = launch("wt.exe", "Enter")
        + &launch("x", "NoSuchKey")
        + "[[launch]]\ncommand = \"\"\nkey = \"Z\"\nmodifiers = []\n";

    // Act
    let lines = launch_lines(&[], entries(&content));

    // Assert
    assert_eq!(lines.len(), 2, "{lines:?}");
    assert!(
        lines[0].contains("launch x has an unknown key: NoSuchKey"),
        "{lines:?}"
    );
    assert!(
        lines[1].contains("launch binding 3: `command` is empty"),
        "{lines:?}"
    );
}

#[test]
fn a_launch_list_that_is_not_tables_fails() {
    // Act
    let lines = launch_lines(&[], entries("launch = 1\n"));

    // Assert
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(lines[0].contains("Launch bindings:"), "{lines:?}");
}

#[test]
fn a_combination_bound_twice_is_named_with_both_bindings() {
    // Arrange
    let keybindings = keybinding::defaults();
    let content =
        launch("wt.exe", "T") + &launch("calc.exe", "Enter") + &launch("notepad.exe", "Enter");

    // Act
    let lines = launch_lines(&keybindings, entries(&content));

    // Assert
    let warnings: Vec<&String> = lines
        .iter()
        .filter(|l| l.contains("bound more than once"))
        .collect();
    assert_eq!(warnings.len(), 2, "{lines:?}");
    assert!(
        warnings[0].contains("Alt+T") && warnings[0].contains("toggle-monocle, launch wt.exe"),
        "{lines:?}"
    );
    assert!(
        warnings[1].contains("Alt+Enter")
            && warnings[1].contains("launch calc.exe, launch notepad.exe"),
        "{lines:?}"
    );
}
