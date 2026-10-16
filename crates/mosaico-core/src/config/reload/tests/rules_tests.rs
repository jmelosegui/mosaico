use crate::config::reload::test_support::{
    INVALID, kinds, rule_classes, rule_for, temp_store, write_at,
};

use super::*;

#[test]
fn a_changed_user_rules_toml_reloads_the_user_rules_ahead_of_the_community_rules() {
    // Arrange
    let store = temp_store("reload-user-rules");
    write_at(&store.rules_path(), &rule_for("Community"), 1);
    write_at(&store.user_rules_path(), "", 1);
    let mut reloader = ConfigReloader::new(store.clone());
    write_at(&store.user_rules_path(), &rule_for("Mine"), 2);

    // Act
    let reloads: Vec<_> = reloader.reload_changed().collect();

    // Assert
    assert_eq!(kinds(&reloads), ["rules"]);
    assert_eq!(rule_classes(&reloads[0]), ["Mine", "Community"]);
}

#[test]
fn an_invalid_user_rules_toml_reloads_the_community_rules_alone() {
    // Arrange
    let store = temp_store("reload-user-rules-invalid");
    write_at(&store.rules_path(), &rule_for("Community"), 1);
    write_at(&store.user_rules_path(), &rule_for("Mine"), 1);
    let mut reloader = ConfigReloader::new(store.clone());
    write_at(&store.user_rules_path(), INVALID, 2);

    // Act
    let reloads: Vec<_> = reloader.reload_changed().collect();

    // Assert -- unlike config.toml and bar.toml, the rules in use do not
    // stay: a file that does not load counts as no user rules, as at
    // startup.
    assert_eq!(kinds(&reloads), ["rules"]);
    assert_eq!(rule_classes(&reloads[0]), ["Community"]);
}

#[test]
fn a_deleted_user_rules_toml_reloads_the_community_rules_alone() {
    // Arrange
    let store = temp_store("reload-user-rules-deleted");
    write_at(&store.rules_path(), &rule_for("Community"), 1);
    write_at(&store.user_rules_path(), &rule_for("Mine"), 1);
    let mut reloader = ConfigReloader::new(store.clone());
    std::fs::remove_file(store.user_rules_path()).unwrap();

    // Act
    let reloads: Vec<_> = reloader.reload_changed().collect();

    // Assert
    assert_eq!(kinds(&reloads), ["rules"]);
    assert_eq!(rule_classes(&reloads[0]), ["Community"]);
}

#[test]
fn a_change_to_rules_toml_alone_reloads_nothing() {
    // Arrange
    let store = temp_store("reload-community-rules");
    write_at(&store.rules_path(), &rule_for("Community"), 1);
    let mut reloader = ConfigReloader::new(store.clone());
    write_at(&store.rules_path(), &rule_for("Downloaded"), 2);

    // Act
    let reloads: Vec<_> = reloader.reload_changed().collect();

    // Assert -- the daemon reloads the community rules itself once it has
    // downloaded them.
    assert!(reloads.is_empty(), "{reloads:?}");
}

#[test]
fn a_change_to_keybindings_toml_reloads_nothing() {
    // Arrange
    let store = temp_store("reload-keybindings");
    let mut reloader = ConfigReloader::new(store.clone());
    write_at(&store.keybindings_path(), "", 1);

    // Act
    let reloads: Vec<_> = reloader.reload_changed().collect();

    // Assert -- hotkeys are registered at startup only.
    assert!(reloads.is_empty(), "{reloads:?}");
}
