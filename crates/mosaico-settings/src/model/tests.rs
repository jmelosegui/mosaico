use std::path::PathBuf;

use mosaico_core::config::{CatppuccinFlavor, ConfigStore, RosePineFlavor};

use super::*;

/// A store on an empty folder of its own, removed when dropped.
struct TempStore(ConfigStore);

impl TempStore {
    fn new(test: &str) -> Self {
        let dir: PathBuf = std::env::temp_dir()
            .join("mosaico-settings-model-tests")
            .join(format!("{test}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Self(ConfigStore::new(dir))
    }

    fn with(test: &str, config: &str) -> Self {
        let store = Self::new(test);
        std::fs::write(store.0.config_path(), config).unwrap();
        store
    }

    fn text(&self) -> String {
        std::fs::read_to_string(self.0.config_path()).unwrap()
    }
}

impl Drop for TempStore {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(self.0.dir());
    }
}

fn on(b: bool) -> SettingValue {
    SettingValue::Bool(b)
}

#[test]
fn each_edited_setting_is_one_change_and_revert_drops_them() {
    // Arrange
    let mut draft = Draft::new(Settings::defaults());

    // Act
    draft.edit.set("logging.enabled", on(true));
    draft.edit.set("logging.max_file_mb", SettingValue::Int(20));
    draft.edit.start_at_sign_in = true;
    let changes = draft.changes();
    draft.revert();

    // Assert
    assert_eq!(changes, 3);
    assert_eq!(draft.changes(), 0);
    assert_eq!(draft.edit, Settings::defaults());
}

#[test]
fn another_theme_family_changes_the_theme_and_the_flavor() {
    let mut draft = Draft::new(Settings::defaults());
    draft
        .edit
        .set_theme(Theme::Catppuccin(CatppuccinFlavor::Latte));
    assert_eq!(draft.changes(), 1);
    draft.edit.set_theme(Theme::RosePine(RosePineFlavor::Dawn));
    assert_eq!(draft.changes(), 2);
}

#[test]
fn the_defaults_are_those_of_a_new_install() {
    let defaults = Settings::defaults();
    assert_eq!(defaults.theme(), Theme::Catppuccin(CatppuccinFlavor::Mocha));
    assert_eq!(defaults.get("logging.enabled"), &on(false));
    assert_eq!(defaults.get("layout.gap"), &SettingValue::Int(20));
    assert!(!defaults.start_at_sign_in);
}

#[test]
fn the_settings_read_are_those_the_file_sets_or_their_defaults() {
    let store = TempStore::with(
        "read",
        "[theme]\nname = \"rose-pine\"\nflavor = \"dawn\"\n[layout]\ngap = 900\n",
    );

    let loaded = Loaded::read(&store.0, true);

    assert_eq!(loaded.problem, None);
    let settings = loaded.settings;
    assert_eq!(settings.theme(), Theme::RosePine(RosePineFlavor::Dawn));
    assert_eq!(settings.get("layout.gap"), &SettingValue::Int(200));
    assert_eq!(settings.get("borders.width"), &SettingValue::Int(4));
    assert!(settings.start_at_sign_in);
}

#[test]
fn saving_writes_only_the_edits_and_keeps_the_rest_of_the_file() {
    // Arrange
    let file = "# mine\n[layout]\ngap = 4 # tight\n\n[mouse]\ndrag_to_swap = false\n";
    let store = TempStore::with("save", file);
    let saved = Loaded::read(&store.0, false).settings;
    let mut edit = saved.clone();

    // Act
    edit.set("layout.gap", SettingValue::Int(12));
    edit.set("mouse.drag_to_swap", on(true));
    edit.set_theme(Theme::Catppuccin(CatppuccinFlavor::Latte));
    let result = save(&store.0, &saved, &edit);

    // Assert
    assert_eq!(result, Ok(()));
    let expected = "# mine\n[layout]\ngap = 12 # tight\n\n[mouse]\n\n[theme]\nflavor = \"latte\"\n";
    assert_eq!(store.text(), expected);
    assert_eq!(Loaded::read(&store.0, false).settings, edit);
}

#[test]
fn a_missing_file_can_be_edited_and_is_created_by_the_first_save() {
    let store = TempStore::new("missing");
    let saved = Loaded::read(&store.0, false);
    let mut edit = saved.settings.clone();
    edit.set("tray.enabled", on(false));

    let result = save(&store.0, &saved.settings, &edit);

    assert_eq!(saved.problem, None);
    assert_eq!(result, Ok(()));
    assert_eq!(store.text(), "[tray]\nenabled = false\n");
}

#[test]
fn a_file_the_daemon_would_not_load_cannot_be_edited_or_saved_over() {
    // Arrange
    let broken = "[animations]\neasing = \"wobble\"\n";
    let store = TempStore::with("broken", broken);

    // Act
    let loaded = Loaded::read(&store.0, false);
    let mut edit = loaded.settings.clone();
    edit.set("layout.gap", SettingValue::Int(1));
    let result = save(&store.0, &loaded.settings, &edit);

    // Assert
    let problem = loaded.problem.unwrap();
    assert!(problem.contains("wobble"), "{problem}");
    assert_eq!(loaded.settings, Settings::defaults());
    assert!(result.unwrap_err().contains("wobble"));
    assert_eq!(store.text(), broken);
}
