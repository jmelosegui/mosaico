use crate::LayoutKind;
use crate::config::store::merge_unfocused::UNFOCUSED_BORDER_HINT_SNIPPET;
use crate::config::store::test_support::{
    cleanup, contents, file_names, temp_store, unique_temp_path,
};
use crate::config::{Config, WorkspaceMode};

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

#[test]
fn merging_sections_without_a_config_file_creates_nothing() {
    // Arrange
    let store = temp_store("merge-sections-missing");

    // Act
    store.merge_missing_config_sections();

    // Assert
    assert!(file_names(&store).is_empty(), "{:?}", file_names(&store));
}

#[test]
fn an_old_config_gets_the_workspaces_section_and_the_unfocused_hint() {
    // Arrange
    let store = temp_store("merge-sections-old");
    let original = "[layout]\ngap = 8\n";
    std::fs::write(store.config_path(), original).unwrap();

    // Act
    store.merge_missing_config_sections();

    // Assert -- the file keeps every byte and gains both blocks, in order.
    assert_eq!(
        contents(&store.config_path()),
        format!("{original}{WORKSPACES_SECTION_SNIPPET}{UNFOCUSED_BORDER_HINT_SNIPPET}")
    );
}

#[test]
fn merging_sections_twice_changes_nothing_the_second_time() {
    // Arrange
    let store = temp_store("merge-sections-twice");
    std::fs::write(store.config_path(), "[layout]\ngap = 8\n").unwrap();
    store.merge_missing_config_sections();
    let merged = contents(&store.config_path());

    // Act
    store.merge_missing_config_sections();

    // Assert
    assert_eq!(contents(&store.config_path()), merged);
}

#[test]
fn an_old_config_is_merged_then_migrated_as_at_daemon_start() {
    // Arrange -- from before [workspaces], with the layouts under [layout].
    let store = temp_store("merge-sections-legacy");
    let original = "[layout.workspaces]\n2 = \"three-column\"\n";
    std::fs::write(store.config_path(), original).unwrap();

    // Act -- in the order the daemon starts.
    store.merge_missing_config_sections();
    let config = store.load_config();

    // Assert -- the old layouts move into the appended section, and the
    // backup is of the file as merged.
    assert_eq!(
        config.workspaces.layouts.get(&2),
        Some(&LayoutKind::ThreeColumn)
    );
    assert_eq!(config.workspaces.mode, WorkspaceMode::PerMonitor);
    let backup = contents(&store.dir().join("config.toml.pre-0.9.bak"));
    assert_eq!(
        backup,
        format!("{original}{WORKSPACES_SECTION_SNIPPET}{UNFOCUSED_BORDER_HINT_SNIPPET}")
    );
}

#[test]
fn an_active_colors_table_gets_the_unfocused_key_even_if_a_comment_names_it() {
    // Arrange -- the user wrote the option down but left it commented out.
    let store = temp_store("merge-sections-colors");
    let original = "[workspaces]\nmode = \"global\"\n\n[borders.colors]\nfocused = \"blue\"\n# unfocused = \"none\"\n";
    std::fs::write(store.config_path(), original).unwrap();

    // Act
    store.merge_missing_config_sections();

    // Assert -- the key goes into the table, the comment stays, nothing
    // is appended and no backup is written.
    let merged = contents(&store.config_path());
    let config: Config = toml::from_str(&merged).unwrap();
    assert_eq!(config.borders.colors.unfocused, "");
    assert!(merged.contains("# unfocused = \"none\""), "{merged}");
    assert!(!merged.contains("Added automatically"), "{merged}");
    assert_eq!(file_names(&store), ["config.toml"]);
}
