use super::*;

#[test]
fn every_file_is_in_the_store_directory() {
    // Arrange
    let dir = Path::new("config-dir");

    // Act
    let store = ConfigStore::new(dir);

    // Assert
    assert_eq!(store.dir(), dir);
    assert_eq!(store.config_path(), dir.join("config.toml"));
    assert_eq!(store.keybindings_path(), dir.join("keybindings.toml"));
    assert_eq!(store.rules_path(), dir.join("rules.toml"));
    assert_eq!(store.user_rules_path(), dir.join("user-rules.toml"));
    assert_eq!(store.bar_path(), dir.join("bar.toml"));
}

#[test]
fn the_store_of_an_instance_is_its_config_directory() {
    // Arrange
    let instance = Instance::new(Some("test"), "config-dir", "data-dir").unwrap();

    // Act
    let store = ConfigStore::for_instance(&instance);

    // Assert
    assert_eq!(store.dir(), instance.config_dir());
}
