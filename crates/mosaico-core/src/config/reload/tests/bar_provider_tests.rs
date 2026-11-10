use crate::config::reload::test_support::{INVALID, kinds, temp_store, write_at};

use super::*;

#[test]
fn bar_toml_is_not_read_while_another_bar_is_picked() {
    // Arrange
    let store = temp_store("reload-bar-other-provider");
    write_at(&store.config_path(), "[bar]\nprovider = \"External\"\n", 1);
    write_at(&store.bar_path(), "", 1);
    let mut reloader = ConfigReloader::new(store.clone());
    write_at(&store.bar_path(), INVALID, 2);

    // Act
    let changed: Vec<_> = reloader.reload_changed().collect();
    let asked: Vec<_> = reloader.load_all();

    // Assert -- not even reported invalid: the file is not in use.
    assert!(changed.is_empty(), "{changed:?}");
    assert_eq!(kinds(&asked), ["config", "rules"]);
}

#[test]
fn bar_toml_is_read_again_once_the_built_in_bar_is_picked() {
    // Arrange
    let store = temp_store("reload-bar-back-to-builtin");
    write_at(&store.config_path(), "[bar]\nprovider = \"none\"\n", 1);
    write_at(&store.bar_path(), "", 1);
    let mut reloader = ConfigReloader::new(store.clone());
    write_at(&store.bar_path(), "height = 30\n", 2);
    let _ = reloader.reload_changed().count();
    write_at(&store.config_path(), "[bar]\nprovider = \"mosaico\"\n", 3);
    write_at(&store.bar_path(), "height = 31\n", 3);

    // Act
    let reloads: Vec<_> = reloader.reload_changed().collect();

    // Assert
    assert_eq!(kinds(&reloads), ["config", "bar"]);
}
