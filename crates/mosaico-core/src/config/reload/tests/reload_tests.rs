use crate::config::reload::test_support::{GAP_7, INVALID, kinds, temp_store, touch_at, write_at};

use super::*;

#[test]
fn nothing_is_reloaded_until_a_file_changes() {
    // Arrange
    let store = temp_store("reload-unchanged");
    write_at(&store.config_path(), GAP_7, 1);
    write_at(&store.user_rules_path(), "", 1);
    write_at(&store.bar_path(), "", 1);
    let mut reloader = ConfigReloader::new(store.clone());

    // Act
    let reloads: Vec<_> = reloader.reload_changed().collect();

    // Assert
    assert!(reloads.is_empty(), "{reloads:?}");
}

#[test]
fn a_changed_config_toml_is_reloaded() {
    // Arrange
    let store = temp_store("reload-config");
    write_at(&store.config_path(), "[layout]\ngap = 3\n", 1);
    let mut reloader = ConfigReloader::new(store.clone());
    write_at(&store.config_path(), GAP_7, 2);

    // Act
    let reloads: Vec<_> = reloader.reload_changed().collect();

    // Assert
    match reloads.as_slice() {
        [ConfigReload::Config(config)] => assert_eq!(config.layout.gap, 7),
        other => panic!("expected a config reload, got {other:?}"),
    }
}

#[test]
fn an_invalid_config_toml_is_skipped_until_its_modification_time_changes_again() {
    // Arrange
    let store = temp_store("reload-config-invalid");
    write_at(&store.config_path(), GAP_7, 1);
    let mut reloader = ConfigReloader::new(store.clone());
    write_at(&store.config_path(), INVALID, 2);

    // Act
    let invalid: Vec<_> = reloader.reload_changed().collect();
    write_at(&store.config_path(), GAP_7, 2);
    let fixed_at_the_same_time: Vec<_> = reloader.reload_changed().collect();
    touch_at(&store.config_path(), 3);
    let touched: Vec<_> = reloader.reload_changed().collect();

    // Assert -- the config in use stays until the file loads, and only
    // the modification time says whether to look again, never content.
    match invalid.as_slice() {
        [ConfigReload::Invalid { file, reason }] => {
            assert_eq!(*file, "config.toml");
            assert!(!reason.is_empty());
        }
        other => panic!("expected config.toml reported invalid, got {other:?}"),
    }
    assert!(
        fixed_at_the_same_time.is_empty(),
        "{fixed_at_the_same_time:?}"
    );
    assert_eq!(kinds(&touched), ["config"]);
}

#[test]
fn a_config_toml_that_appears_is_loaded() {
    // Arrange
    let store = temp_store("reload-config-created");
    let mut reloader = ConfigReloader::new(store.clone());
    write_at(&store.config_path(), GAP_7, 1);

    // Act
    let reloads: Vec<_> = reloader.reload_changed().collect();

    // Assert
    assert_eq!(kinds(&reloads), ["config"]);
}

#[test]
fn a_deleted_config_toml_keeps_the_config_in_use() {
    // Arrange
    let store = temp_store("reload-config-deleted");
    write_at(&store.config_path(), GAP_7, 1);
    let mut reloader = ConfigReloader::new(store.clone());
    std::fs::remove_file(store.config_path()).unwrap();

    // Act
    let reloads: Vec<_> = reloader.reload_changed().collect();

    // Assert -- a missing file does not load, so nothing replaces the
    // config in use, the defaults included.
    assert!(reloads.is_empty(), "{reloads:?}");
}

#[test]
fn a_changed_bar_toml_is_reloaded() {
    // Arrange
    let store = temp_store("reload-bar");
    write_at(&store.bar_path(), "height = 30\n", 1);
    let mut reloader = ConfigReloader::new(store.clone());
    write_at(&store.bar_path(), "height = 40\n", 2);

    // Act
    let reloads: Vec<_> = reloader.reload_changed().collect();

    // Assert
    match reloads.as_slice() {
        [ConfigReload::Bar(bar)] => assert_eq!(bar.height, 40),
        other => panic!("expected a bar reload, got {other:?}"),
    }
}

#[test]
fn an_invalid_bar_toml_keeps_the_bar_in_use() {
    // Arrange
    let store = temp_store("reload-bar-invalid");
    write_at(&store.bar_path(), "height = 30\n", 1);
    let mut reloader = ConfigReloader::new(store.clone());
    write_at(&store.bar_path(), INVALID, 2);

    // Act
    let reloads: Vec<_> = reloader.reload_changed().collect();

    // Assert -- reported, with no bar to replace the one in use.
    assert_eq!(kinds(&reloads), ["invalid"]);
}

#[test]
fn load_all_loads_every_file_unchanged_or_not() {
    // Arrange
    let store = temp_store("reload-all");
    write_at(&store.config_path(), GAP_7, 1);
    write_at(&store.bar_path(), INVALID, 1);
    let mut reloader = ConfigReloader::new(store.clone());

    // Act
    let loaded = reloader.load_all();
    let after: Vec<_> = reloader.reload_changed().collect();

    // Assert -- the bar is reported, and nothing is pending afterwards.
    assert_eq!(kinds(&loaded), ["config", "rules", "invalid"]);
    assert!(after.is_empty(), "{after:?}");
}

#[test]
fn load_all_skips_a_file_that_does_not_exist() {
    // Arrange
    let store = temp_store("reload-all-missing");
    let mut reloader = ConfigReloader::new(store.clone());

    // Act
    let loaded = reloader.load_all();

    // Assert -- the rules always load, user rules or not.
    assert_eq!(kinds(&loaded), ["rules"]);
}

#[test]
fn a_file_that_cannot_be_read_is_skipped_not_reported() {
    // Arrange -- a directory stands in for a file another program holds
    // open while it writes it.
    let store = temp_store("reload-unreadable");
    let mut reloader = ConfigReloader::new(store.clone());
    std::fs::create_dir(store.config_path()).unwrap();

    // Act
    let reloads: Vec<_> = reloader.reload_changed().collect();

    // Assert
    assert_eq!(kinds(&reloads), Vec::<&str>::new());
}

#[test]
fn an_invalid_file_is_reported_without_its_path() {
    // Arrange
    let store = temp_store("reload-invalid-reason");
    let mut reloader = ConfigReloader::new(store.clone());
    write_at(&store.config_path(), "this is = = broken", 1);

    // Act
    let reloads: Vec<_> = reloader.reload_changed().collect();

    // Assert
    let [ConfigReload::Invalid { file, reason }] = reloads.as_slice() else {
        panic!("expected one invalid file, got {reloads:?}");
    };
    assert_eq!(*file, "config.toml");
    assert!(reason.starts_with("TOML parse error"), "{reason}");
}

#[test]
fn several_changed_files_reload_in_the_order_config_rules_bar() {
    // Arrange
    let store = temp_store("reload-order");
    let mut reloader = ConfigReloader::new(store.clone());
    write_at(&store.bar_path(), "", 1);
    write_at(&store.user_rules_path(), "", 1);
    write_at(&store.config_path(), GAP_7, 1);

    // Act
    let reloads: Vec<_> = reloader.reload_changed().collect();

    // Assert
    assert_eq!(kinds(&reloads), ["config", "rules", "bar"]);
}

#[test]
fn a_file_the_caller_did_not_reach_is_still_pending() {
    // Arrange
    let store = temp_store("reload-pending");
    let mut reloader = ConfigReloader::new(store.clone());
    write_at(&store.config_path(), GAP_7, 1);
    write_at(&store.bar_path(), "", 1);

    // Act -- the watch stops reading at the first reload it cannot send.
    let first: Vec<_> = reloader.reload_changed().take(1).collect();
    let rest: Vec<_> = reloader.reload_changed().collect();

    // Assert
    assert_eq!(kinds(&first), ["config"]);
    assert_eq!(kinds(&rest), ["bar"]);
}
