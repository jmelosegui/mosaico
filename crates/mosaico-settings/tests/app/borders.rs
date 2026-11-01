//! The Borders page and its color picker.

use crate::support::*;

/// A window on the Borders page, and where its config folder is.
fn borders(test: &str) -> (Harness<'static, SettingsApp>, PathBuf) {
    let store = folder(test);
    let dir = store.config_path().parent().unwrap().to_path_buf();
    let mut h = window(store, DaemonState::Stopped, &SignIn::default());
    open(&mut h, "Borders");
    (h, dir)
}

fn written(dir: &std::path::Path) -> String {
    std::fs::read_to_string(dir.join("config.toml")).unwrap()
}

fn apply(h: &mut Harness<'static, SettingsApp>) {
    click(h, Role::Button, "Apply");
}

fn text(h: &Harness<'static, SettingsApp>, key: &str) -> String {
    h.state().edit().get(key).as_text().unwrap().to_owned()
}

#[test]
fn a_palette_color_picked_for_focused_is_saved_by_name() {
    let (mut h, dir) = borders("borders-palette");

    click(&mut h, Role::Button, "Focused");
    click(&mut h, Role::Button, "mauve");
    apply(&mut h);

    assert_eq!(written(&dir), "[borders.colors]\nfocused = \"mauve\"\n");
}

#[test]
fn the_palette_offered_is_that_of_the_theme_being_edited() {
    let (mut h, _) = borders("borders-theme");
    open(&mut h, "General");
    click(&mut h, Role::ComboBox, "Theme");
    click(&mut h, Role::Button, "Rosé Pine");
    open(&mut h, "Borders");

    click(&mut h, Role::Button, "Focused");

    h.get_by_role_and_label(Role::Button, "iris");
    assert!(
        h.query_by_role_and_label(Role::Button, "rosewater")
            .is_none(),
        "rosewater is not a Rosé Pine color"
    );
}

#[test]
fn a_hex_typed_is_saved() {
    let (mut h, dir) = borders("borders-hex");

    click(&mut h, Role::Button, "Focused");
    h.get_by_role_and_label(Role::TextInput, "Focused hex")
        .click();
    h.run();
    h.get_by_role_and_label(Role::TextInput, "Focused hex")
        .type_text("#ff8800");
    h.run();
    apply(&mut h);

    assert_eq!(written(&dir), "[borders.colors]\nfocused = \"#ff8800\"\n");
}

#[test]
fn a_hex_that_is_no_color_is_not_taken() {
    let (mut h, _) = borders("borders-bad-hex");

    click(&mut h, Role::Button, "Focused");
    h.get_by_role_and_label(Role::TextInput, "Focused hex")
        .click();
    h.run();
    h.get_by_role_and_label(Role::TextInput, "Focused hex")
        .type_text("#ff88");
    h.run();

    assert_eq!(text(&h, "borders.colors.focused"), "");
    h.get_by_label("Use six hex digits, like #89b4fa.");
    h.get_by_label("No unsaved changes");
}

#[test]
fn none_is_offered_for_unfocused_only_and_saved() {
    let (mut h, dir) = borders("borders-none");
    click(&mut h, Role::Button, "Focused");
    assert!(h.query_by_role_and_label(Role::Button, "None").is_none());
    h.key_press(eframe::egui::Key::Escape);
    h.run();

    click(&mut h, Role::Button, "Unfocused");
    click(&mut h, Role::Button, "None");
    apply(&mut h);

    assert_eq!(written(&dir), "[borders.colors]\nunfocused = \"none\"\n");
}

#[test]
fn theme_default_removes_the_key() {
    let (mut h, dir) = borders("borders-default");
    click(&mut h, Role::Button, "Focused");
    click(&mut h, Role::Button, "mauve");
    apply(&mut h);

    click(&mut h, Role::Button, "Focused");
    click(&mut h, Role::Button, "Theme default");
    apply(&mut h);

    assert!(!written(&dir).contains("focused"));
}

#[test]
fn width_and_corners_are_saved() {
    let (mut h, dir) = borders("borders-width");

    h.get_by_role_and_label(Role::SpinButton, "Width").click();
    h.run();
    h.get_by_role_and_label(Role::SpinButton, "Width")
        .type_text("6");
    h.key_press(eframe::egui::Key::Enter);
    h.run();
    click(&mut h, Role::RadioButton, "Round");
    apply(&mut h);

    let file = written(&dir);
    assert!(file.contains("width = 6"), "{file}");
    assert!(file.contains("corner_style = \"round\""), "{file}");
}
