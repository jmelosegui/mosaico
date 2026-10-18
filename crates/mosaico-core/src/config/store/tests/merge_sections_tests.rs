use crate::LayoutKind;
use crate::config::store::merge_animations::ANIMATIONS_HINT_SNIPPET;
use crate::config::store::merge_mouse::{DRAG_TO_SWAP_HINT_SNIPPET, RESIZE_SPLITS_HINT_SNIPPET};
use crate::config::store::merge_unfocused::UNFOCUSED_BORDER_HINT_SNIPPET;
use crate::config::store::merge_wrap::WRAP_INLINE_PREFIX;
use crate::config::store::test_support::{contents, file_names, temp_store};
use crate::config::{Config, WorkspaceMode};

use super::*;

/// `original` with everything a config that has none of it gains, in
/// the order it is added.
fn fully_merged(original: &str) -> String {
    [
        original,
        WORKSPACES_SECTION_SNIPPET,
        UNFOCUSED_BORDER_HINT_SNIPPET,
        DRAG_TO_SWAP_HINT_SNIPPET,
        RESIZE_SPLITS_HINT_SNIPPET,
        ANIMATIONS_HINT_SNIPPET,
    ]
    .concat()
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
fn an_old_config_gets_the_workspaces_section_and_the_hints() {
    // Arrange
    let store = temp_store("merge-sections-old");
    let original = "[layout]\ngap = 8\n";
    std::fs::write(store.config_path(), original).unwrap();

    // Act
    store.merge_missing_config_sections();

    // Assert -- the file keeps every byte and gains the five blocks, in
    // order.
    assert_eq!(contents(&store.config_path()), fully_merged(original));
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
    assert_eq!(backup, fully_merged(original));
}

#[test]
fn an_active_colors_table_gets_the_unfocused_key_even_if_a_comment_names_it() {
    // Arrange -- the user wrote the option down but left it commented out.
    let store = temp_store("merge-sections-colors");
    let original = "[workspaces]\nmode = \"global\"\n\n[borders.colors]\nfocused = \"blue\"\n# unfocused = \"none\"\n";
    std::fs::write(store.config_path(), original).unwrap();

    // Act
    store.merge_missing_config_sections();

    // Assert -- the key goes into the table, the comment stays, the
    // unfocused hint is not appended and no backup is written.
    let merged = contents(&store.config_path());
    let config: Config = toml::from_str(&merged).unwrap();
    assert_eq!(config.borders.colors.unfocused, "");
    assert!(merged.contains("# unfocused = \"none\""), "{merged}");
    assert!(!merged.contains(UNFOCUSED_BORDER_HINT_SNIPPET), "{merged}");
    assert_eq!(file_names(&store), ["config.toml"]);
}

#[test]
fn a_config_with_every_section_gains_nothing() {
    // Arrange -- the near miss: a file that already has all of it.
    let store = temp_store("merge-sections-current");
    let original = "[workspaces]\nmode = \"global\"\nwrap = true\n\n[borders.colors]\nunfocused = \"none\"\n\n[mouse]\ndrag_to_swap = true\nresize_splits = true\n\n[animations]\nenabled = true\n";
    std::fs::write(store.config_path(), original).unwrap();

    // Act
    store.merge_missing_config_sections();

    // Assert
    assert_eq!(contents(&store.config_path()), original);
    assert_eq!(file_names(&store), ["config.toml"]);
}

#[test]
fn a_workspaces_section_from_before_wrap_gains_the_key_in_place() {
    // Arrange -- everything else is there already.
    let store = temp_store("merge-sections-wrap");
    let workspaces = "[workspaces]\nmode = \"global\"\n";
    let rest = "\n[borders.colors]\nunfocused = \"none\"\n\n[mouse]\ndrag_to_swap = true\nresize_splits = true\n\n[animations]\nenabled = true\n";
    std::fs::write(store.config_path(), format!("{workspaces}{rest}")).unwrap();

    // Act
    store.merge_missing_config_sections();

    // Assert -- under its comment, at the end of the section, with no
    // backup written.
    let merged = format!("{workspaces}{WRAP_INLINE_PREFIX}wrap = false\n{rest}");
    assert_eq!(contents(&store.config_path()), merged);
    assert_eq!(file_names(&store), ["config.toml"]);
}
