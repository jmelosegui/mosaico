//! The Layout page.

use crate::support::*;

#[test]
fn the_layout_page_saves_its_settings_and_shows_them_when_reopened() {
    // Arrange
    let store = folder("layout-page");
    let dir = store.config_path().parent().unwrap().to_path_buf();
    let mut h = window(store, DaemonState::Stopped, &SignIn::default());
    open(&mut h, "Layout");

    // Act
    click(&mut h, Role::RadioButton, "Vertical stack");
    click(&mut h, Role::RadioButton, "Minimize");
    click(&mut h, Role::Button, "Apply");

    // Assert
    let written = std::fs::read_to_string(dir.join("config.toml")).unwrap();
    assert_eq!(
        written,
        "[layout]\ndefault = \"vertical-stack\"\nhiding = \"minimize\"\n"
    );
    let mut again = window(
        ConfigStore::new(dir),
        DaemonState::Stopped,
        &SignIn::default(),
    );
    open(&mut again, "Layout");
    let edit = again.state().edit();
    assert_eq!(
        edit.get("layout.default"),
        &SettingValue::Text("vertical-stack".into())
    );
    assert!(again.query_by_label("2 unsaved changes").is_none());
}

#[test]
fn a_number_typed_on_the_layout_page_is_an_edit() {
    let mut h = stopped();
    open(&mut h, "Layout");

    h.get_by_role_and_label(Role::SpinButton, "Gap").click();
    h.run();
    h.get_by_role_and_label(Role::SpinButton, "Gap")
        .type_text("1");
    h.key_press(eframe::egui::Key::Enter);
    h.run();

    assert_ne!(
        h.state().edit().get("layout.gap"),
        h.state().saved().get("layout.gap")
    );
    h.get_by_label("1 unsaved change");
}
