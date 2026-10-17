use crate::config::store::test_support::{cleanup, unique_temp_path};
use crate::config::{Config, WorkspaceMode};

use super::*;

/// Writes `original` to a file of its own, adds `wrap` to it, and returns
/// whether that changed it and what it holds then.
fn added(original: &str) -> (bool, String) {
    let path = unique_temp_path();
    std::fs::write(&path, original).unwrap();
    let changed = add_missing_wrap_key(&path).unwrap();
    let after = std::fs::read_to_string(&path).unwrap();
    cleanup(&path);
    (changed, after)
}

#[test]
fn an_active_workspaces_table_gets_the_key_under_a_comment() {
    // Arrange
    let original = "[workspaces]\nmode = \"global\"\n";

    // Act
    let (changed, after) = added(original);

    // Assert -- after what the table had, and with the default value.
    assert!(changed);
    assert_eq!(
        after,
        format!("{original}{WRAP_INLINE_PREFIX}wrap = false\n")
    );
    let config: Config = toml::from_str(&after).unwrap();
    assert_eq!(config.workspaces.mode, WorkspaceMode::Global);
    assert!(!config.workspaces.wrap);
}

#[test]
fn the_key_goes_in_the_table_ahead_of_the_layouts() {
    // Arrange
    let original = "[workspaces]\nmode = \"global\"\n\n[workspaces.layouts]\n2 = \"bsp\"\n";

    // Act
    let (_, after) = added(original);

    // Assert
    let wrap = after.find("wrap = false").unwrap();
    assert!(
        wrap < after.find("[workspaces.layouts]").unwrap(),
        "{after}"
    );
    let config: Config = toml::from_str(&after).unwrap();
    assert_eq!(config.workspaces.layouts.len(), 1);
}

#[test]
fn adding_the_key_twice_changes_nothing_the_second_time() {
    // Arrange
    let path = unique_temp_path();
    std::fs::write(&path, "[workspaces]\nmode = \"global\"\n").unwrap();
    add_missing_wrap_key(&path).unwrap();
    let first = std::fs::read_to_string(&path).unwrap();

    // Act
    let changed = add_missing_wrap_key(&path).unwrap();

    // Assert
    assert!(!changed);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), first);
    cleanup(&path);
}

#[test]
fn a_table_that_has_the_key_is_left_alone() {
    // Arrange
    let original = "[workspaces]\nwrap = true\n";

    // Act
    let (changed, after) = added(original);

    // Assert
    assert!(!changed);
    assert_eq!(after, original);
}

#[test]
fn a_section_in_another_form_gets_the_comment_block() {
    // Arrange -- an inline table, and one only its layouts make.
    let inline = "workspaces = { mode = \"global\" }\n";
    let implicit = "[workspaces.layouts]\n2 = \"bsp\"\n";

    // Act
    let results = [added(inline), added(implicit)];

    // Assert -- the files are as they were, but for the block.
    assert_eq!(results[0], (true, format!("{inline}{WRAP_HINT_SNIPPET}")));
    assert_eq!(results[1], (true, format!("{implicit}{WRAP_HINT_SNIPPET}")));
}

#[test]
fn a_section_in_another_form_that_mentions_wrap_is_left_alone() {
    // Arrange
    let original = "workspaces = { mode = \"global\" }\n# wrap = true\n";

    // Act
    let (changed, after) = added(original);

    // Assert
    assert!(!changed);
    assert_eq!(after, original);
}

#[test]
fn a_config_without_the_section_is_left_alone() {
    // Arrange -- the section appended to it has the key.
    let original = "[layout]\ngap = 8\n";

    // Act
    let (changed, after) = added(original);

    // Assert
    assert!(!changed);
    assert_eq!(after, original);
}
