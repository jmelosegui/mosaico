use crate::config::store::test_support::{contents, file_names, temp_store};
use crate::config::{Modifier, WorkspaceMode};
use crate::{Action, Error, LayoutKind};

use super::*;

#[test]
fn missing_files_load_as_the_defaults_and_nothing_is_created() {
    // Arrange
    let store = temp_store("load-missing");

    // Act
    let config = store.load_config();
    let keybindings = store.load_keybindings();
    let rules = store.load_merged_rules();
    let bar = store.load_bar();

    // Assert
    assert_eq!(config.layout.gap, Config::default().layout.gap);
    assert_eq!(keybindings.len(), keybinding::defaults().len());
    assert!(rules.is_empty());
    assert_eq!(bar.height, BarConfig::default().height);
    assert!(file_names(&store).is_empty(), "{:?}", file_names(&store));
}

#[test]
fn a_missing_file_is_a_config_error_that_names_it() {
    // Arrange
    let store = temp_store("load-missing-error");

    // Act
    let results = [
        (store.config_path(), store.try_load_config().map(|_| ())),
        (
            store.keybindings_path(),
            store.try_load_keybindings().map(|_| ()),
        ),
        (store.rules_path(), store.try_load_rules().map(|_| ())),
        (
            store.user_rules_path(),
            store.try_load_user_rules().map(|_| ()),
        ),
        (store.bar_path(), store.try_load_bar().map(|_| ())),
    ];

    // Assert
    for (path, result) in results {
        match result {
            Err(Error::Config {
                path: Some(named), ..
            }) => assert_eq!(named, path),
            other => panic!("{}: expected a config error, got {other:?}", path.display()),
        }
    }
}

#[test]
fn an_invalid_file_is_an_error_and_loads_as_the_defaults() {
    // Arrange
    let store = temp_store("load-invalid");
    let files = [
        store.config_path(),
        store.keybindings_path(),
        store.rules_path(),
        store.user_rules_path(),
        store.bar_path(),
    ];
    for path in &files {
        std::fs::write(path, "not = [valid").unwrap();
    }

    // Act
    let error = store.try_load_config().unwrap_err().to_string();
    let config = store.load_config();
    let keybindings = store.load_keybindings();
    let rules = store.load_merged_rules();
    let bar = store.load_bar();

    // Assert -- the message starts with the file, as doctor shows it.
    let prefix = format!("{}: ", store.config_path().display());
    assert!(error.starts_with(&prefix), "{error}");
    assert!(store.try_load_keybindings().is_err());
    assert!(store.try_load_rules().is_err());
    assert!(store.try_load_user_rules().is_err());
    assert!(store.try_load_bar().is_err());
    assert_eq!(config.layout.gap, Config::default().layout.gap);
    assert_eq!(keybindings.len(), keybinding::defaults().len());
    assert!(rules.is_empty());
    assert_eq!(bar.height, BarConfig::default().height);
    for path in &files {
        assert_eq!(
            contents(path),
            "not = [valid",
            "{} was changed",
            path.display()
        );
    }
}

#[test]
fn loaded_values_are_validated_and_the_files_left_alone() {
    // Arrange
    let store = temp_store("load-validate");
    std::fs::write(store.config_path(), "[layout]\ngap = 500\n").unwrap();
    std::fs::write(store.bar_path(), "height = 500\n").unwrap();

    // Act
    let config = store.try_load_config().unwrap();
    let bar = store.try_load_bar().unwrap();

    // Assert
    assert_eq!(config.layout.gap, 200);
    assert_eq!(config.borders.colors.focused, "#89b4fa", "from the theme");
    assert_eq!(bar.height, 96);
    assert_eq!(file_names(&store), ["bar.toml", "config.toml"]);
    assert_eq!(contents(&store.config_path()), "[layout]\ngap = 500\n");
}

#[test]
fn keybindings_in_the_file_replace_the_defaults() {
    // Arrange
    let store = temp_store("load-keybindings");
    let file = "[[keybinding]]\naction = \"retile\"\nkey = \"F5\"\nmodifiers = [\"ctrl\"]\n";
    std::fs::write(store.keybindings_path(), file).unwrap();

    // Act
    let bindings = store.load_keybindings();

    // Assert
    assert_eq!(bindings.len(), 1);
    assert_eq!(bindings[0].action, Action::Retile);
    assert_eq!(bindings[0].key, "F5");
    assert_eq!(bindings[0].modifiers, [Modifier::Ctrl]);
}

#[test]
fn user_rules_come_before_community_rules() {
    // Arrange
    let store = temp_store("load-rules");
    let community = "[[rule]]\nmatch_class = \"Community\"\nmanage = false\n";
    let user = "[[rule]]\nmatch_class = \"Mine\"\nmanage = true\n";
    std::fs::write(store.rules_path(), community).unwrap();
    std::fs::write(store.user_rules_path(), user).unwrap();

    // Act
    let rules = store.load_merged_rules();

    // Assert
    let classes: Vec<_> = rules.iter().map(|r| r.match_class.as_deref()).collect();
    assert_eq!(classes, [Some("Mine"), Some("Community")]);
}

#[test]
fn loading_an_old_config_backs_it_up_and_rewrites_it() {
    // Arrange
    let store = temp_store("load-migrate");
    let original =
        "[layout.workspaces]\n1 = \"vertical-stack\"\n\n[borders]\nfocused = \"#ff0000\"\n";
    std::fs::write(store.config_path(), original).unwrap();

    // Act
    let config = store.try_load_config().unwrap();

    // Assert
    assert_eq!(
        config.workspaces.layouts.get(&1),
        Some(&LayoutKind::VerticalStack)
    );
    assert_eq!(config.workspaces.mode, WorkspaceMode::PerMonitor);
    assert_eq!(config.borders.colors.focused, "#ff0000");
    assert_eq!(
        file_names(&store),
        ["config.toml", "config.toml.pre-0.9.bak"]
    );
    let backup = store.dir().join("config.toml.pre-0.9.bak");
    assert_eq!(contents(&backup), original);
    let rewritten = contents(&store.config_path());
    assert!(rewritten.contains("[workspaces.layouts]"), "{rewritten}");
    assert!(!rewritten.contains("[layout.workspaces]"), "{rewritten}");
}

#[test]
fn an_earlier_backup_is_never_overwritten() {
    // Arrange
    let store = temp_store("load-migrate-backup");
    let earlier = store.dir().join("config.toml.pre-0.9.bak");
    std::fs::write(&earlier, "an earlier backup").unwrap();
    let original = "[layout.workspaces]\n1 = \"bsp\"\n";
    std::fs::write(store.config_path(), original).unwrap();

    // Act
    store.load_config();

    // Assert
    assert_eq!(contents(&earlier), "an earlier backup");
    let backup = store.dir().join("config.toml.pre-0.9.bak.1");
    assert_eq!(contents(&backup), original);
}

#[test]
fn a_migrated_config_loads_again_without_another_backup() {
    // Arrange
    let store = temp_store("load-migrate-twice");
    std::fs::write(store.config_path(), "[borders]\nmonocle = \"#00ff00\"\n").unwrap();
    store.load_config();
    let migrated = contents(&store.config_path());

    // Act
    let config = store.load_config();

    // Assert
    assert_eq!(config.borders.colors.monocle, "#00ff00");
    assert_eq!(contents(&store.config_path()), migrated);
    assert_eq!(
        file_names(&store),
        ["config.toml", "config.toml.pre-0.9.bak"]
    );
}
