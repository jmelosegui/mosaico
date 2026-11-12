//! Files seen in the middle of a save that truncates them and writes them
//! again.

use crate::config::reload::test_support::{GAP_7, temp_store, write_at};

use super::*;

#[test]
fn a_write_that_keeps_the_modification_time_is_still_reloaded() {
    // Arrange -- a save truncates the file and then writes it, and both
    // can carry the same modification time. Seen in between, the file
    // holds only part of what is being saved.
    let store = temp_store("reload-same-time");
    write_at(&store.config_path(), "[layout]\ngap = 3\n", 1);
    let mut reloader = ConfigReloader::new(store.clone());
    write_at(&store.config_path(), "[layout]\n", 2);
    let _partly_written: Vec<_> = reloader.reload_changed().collect();
    write_at(&store.config_path(), GAP_7, 2);

    // Act
    let reloads: Vec<_> = reloader.reload_changed().collect();

    // Assert -- what the save wrote is in use, not what was seen halfway.
    match reloads.as_slice() {
        [ConfigReload::Config(config)] => assert_eq!(config.layout.gap, 7),
        other => panic!("expected the saved config reloaded, got {other:?}"),
    }
}

#[test]
fn an_emptied_config_toml_is_taken_as_one_being_written() {
    // Arrange -- the first step of a save that truncates the file.
    let store = temp_store("reload-emptied");
    write_at(&store.config_path(), GAP_7, 1);
    let mut reloader = ConfigReloader::new(store.clone());
    write_at(&store.config_path(), "", 2);

    // Act
    let reloads: Vec<_> = reloader.reload_changed().collect();

    // Assert -- the defaults do not replace the config in use while the
    // save is under way.
    assert!(reloads.is_empty(), "{reloads:?}");
}

#[test]
fn load_all_loads_an_empty_config_toml_as_the_defaults() {
    // Arrange
    let store = temp_store("reload-all-empty");
    write_at(&store.config_path(), "", 1);
    let mut reloader = ConfigReloader::new(store.clone());

    // Act -- the user asks for the files to be read as they are.
    let loaded = reloader.load_all();

    // Assert
    match loaded.first() {
        Some(ConfigReload::Config(config)) => {
            assert_eq!(config.layout.gap, Config::default().layout.gap)
        }
        other => panic!("expected the default config, got {other:?}"),
    }
}
