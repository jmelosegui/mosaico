use super::*;

fn store(test: &str) -> ConfigStore {
    let dir = std::env::temp_dir().join(format!("mosaico-settings-app-{test}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    ConfigStore::new(dir)
}

#[test]
fn what_the_window_saves_is_what_it_loads_next_time() {
    // Arrange
    let store = store("round-trip");
    let app = SettingsAppConfig {
        page: "Bar".into(),
        width: 1200.0,
        height: 800.0,
        maximized: true,
    };

    // Act
    store.save_settings_app(&app).unwrap();

    // Assert
    assert_eq!(store.load_settings_app(), app);
    let _ = std::fs::remove_dir_all(store.dir());
}

#[test]
fn the_window_saves_into_a_config_folder_that_does_not_exist_yet() {
    // Arrange
    let store = store("no-folder");
    std::fs::remove_dir_all(store.dir()).unwrap();
    let app = SettingsAppConfig {
        page: "Bar".into(),
        ..SettingsAppConfig::default()
    };

    // Act
    let saved = store.save_settings_app(&app);

    // Assert
    assert!(saved.is_ok(), "{saved:?}");
    assert_eq!(store.load_settings_app(), app);
    let _ = std::fs::remove_dir_all(store.dir());
}

#[test]
fn a_missing_or_broken_file_gives_the_defaults() {
    let store = store("broken");
    assert_eq!(store.load_settings_app(), SettingsAppConfig::default());

    std::fs::write(store.settings_app_path(), "width = = 3").unwrap();

    assert_eq!(store.load_settings_app(), SettingsAppConfig::default());
    let _ = std::fs::remove_dir_all(store.dir());
}

#[test]
fn a_remembered_size_is_kept_between_the_least_and_the_most() {
    let at = |width, height| SettingsAppConfig {
        width,
        height,
        ..SettingsAppConfig::default()
    };

    assert_eq!(at(1200.0, 800.0).size(), (1200.0, 800.0));
    assert_eq!(at(10.0, -5.0).size(), SETTINGS_WINDOW_MIN);
    assert_eq!(at(1e9, f32::NAN).size(), (16_384.0, SETTINGS_WINDOW_SIZE.1));
}

#[test]
fn saving_what_the_file_already_says_leaves_it_alone() {
    // Arrange
    let store = store("unchanged");
    let path = store.settings_app_path();
    std::fs::write(&path, "# mine\npage = \"Bar\"\n").unwrap();
    let app = store.load_settings_app();

    // Act
    store.save_settings_app(&app).unwrap();

    // Assert
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "# mine\npage = \"Bar\"\n"
    );
    let _ = std::fs::remove_dir_all(store.dir());
}
