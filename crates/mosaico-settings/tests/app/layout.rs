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
    click(&mut h, Role::ComboBox, "Hide windows on other workspaces");
    // The title bar has a Minimize button too; the option is below it.
    h.get_all_by_label("Minimize")
        .max_by(|a, b| a.rect().top().total_cmp(&b.rect().top()))
        .expect("the option in the drop-down")
        .click();
    h.run();
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

#[test]
fn a_gap_beyond_the_end_of_the_track_can_be_typed() {
    let mut h = stopped();
    open(&mut h, "Layout");

    h.get_by_role_and_label(Role::SpinButton, "Gap").click();
    h.run();
    let field = h.get_by_role_and_label(Role::SpinButton, "Gap");
    field.type_text("120");
    h.key_press(eframe::egui::Key::Enter);
    h.run();

    assert_eq!(h.state().edit().get("layout.gap"), &SettingValue::Int(120));
}

#[test]
fn clicking_the_far_end_of_the_gap_track_sets_the_largest_usual_gap() {
    let mut h = stopped();
    open(&mut h, "Layout");

    let track = h.get_by_role_and_label(Role::Slider, "Gap").rect();
    h.hover_at(track.right_center() - eframe::egui::vec2(1.0, 0.0));
    h.run();
    h.event(eframe::egui::Event::PointerButton {
        pos: track.right_center() - eframe::egui::vec2(1.0, 0.0),
        button: eframe::egui::PointerButton::Primary,
        pressed: true,
        modifiers: Default::default(),
    });
    h.run();
    h.event(eframe::egui::Event::PointerButton {
        pos: track.right_center() - eframe::egui::vec2(1.0, 0.0),
        button: eframe::egui::PointerButton::Primary,
        pressed: false,
        modifiers: Default::default(),
    });
    h.run();

    assert_eq!(h.state().edit().get("layout.gap"), &SettingValue::Int(48));
}
