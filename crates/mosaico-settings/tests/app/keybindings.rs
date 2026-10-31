//! The Keybindings page.

use eframe::egui::{Key, Modifiers};

use crate::support::*;

const FILE: &str = "# my keys\n\n[[keybinding]]\naction = \"focus-down\" # down\nkey = \"J\"\nmodifiers = [\"alt\"]\n\n[[keybinding]]\naction = \"focus-up\"\nkey = \"K\"\nmodifiers = [\"alt\"]\n";

fn window_on(test: &str) -> (Harness<'static, SettingsApp>, PathBuf) {
    let store = folder(test);
    let path = store.keybindings_path();
    std::fs::write(&path, FILE).unwrap();
    let mut h = window(store, DaemonState::Stopped, &SignIn::default());
    open(&mut h, "Keybindings");
    (h, path)
}

fn press(h: &mut Harness<'static, SettingsApp>, modifiers: Modifiers, key: Key) {
    h.key_press_modifiers(modifiers, key);
    h.run();
}

#[test]
fn a_captured_combination_is_saved_and_keeps_the_comments() {
    // Arrange
    let (mut h, path) = window_on("keys-capture");
    click(&mut h, Role::Button, "Change Focus down");
    h.get_by_label("Press the new key combination, or Esc to cancel.");

    // Act
    press(&mut h, Modifiers::ALT | Modifiers::SHIFT, Key::Y);

    // Assert
    h.get_by_label("1 unsaved change");
    assert!(
        h.query_by_label("Press the new key combination, or Esc to cancel.")
            .is_none()
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), FILE);
    click(&mut h, Role::Button, "Apply");
    let expected =
        FILE.replace("key = \"J\"", "key = \"Y\"")
            .replacen("[\"alt\"]", "[\"alt\", \"shift\"]", 1);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), expected);
    h.get_by_label("Changes saved");
}

#[test]
fn escape_cancels_a_capture() {
    let (mut h, path) = window_on("keys-escape");
    click(&mut h, Role::Button, "Change Focus down");

    press(&mut h, Modifiers::NONE, Key::Escape);

    assert!(
        h.query_by_label("Press the new key combination, or Esc to cancel.")
            .is_none()
    );
    h.get_by_label("No unsaved changes");
    assert_eq!(std::fs::read_to_string(path).unwrap(), FILE);
}

#[test]
fn a_press_without_a_modifier_is_refused_and_the_capture_goes_on() {
    let (mut h, _) = window_on("keys-bare");
    click(&mut h, Role::Button, "Change Focus down");

    press(&mut h, Modifiers::NONE, Key::Y);

    h.get_by_label("Hold Ctrl, Alt or Shift while you press the key.");
    h.get_by_label("No unsaved changes");
}

#[test]
fn a_key_mosaico_has_no_name_for_is_refused() {
    let (mut h, _) = window_on("keys-unknown");
    click(&mut h, Role::Button, "Change Focus down");

    press(&mut h, Modifiers::ALT, Key::Colon);

    h.get_by_label("Colon can't be used as a hotkey.");
    h.get_by_label("No unsaved changes");
}

#[test]
fn a_combination_bound_twice_is_flagged_and_not_saved() {
    // Arrange
    let (mut h, path) = window_on("keys-conflict");
    click(&mut h, Role::Button, "Change Focus up");

    // Act
    press(&mut h, Modifiers::ALT, Key::J);
    click(&mut h, Role::Button, "Apply");

    // Assert
    assert_eq!(h.query_all_by_label("Conflict").count(), 2);
    h.get_by_label_contains("Also bound to Focus down.");
    h.get_by_label_contains("Also bound to Focus up.");
    assert!(matches!(h.state().applied(), Some(Applied::Failed(_))));
    assert_eq!(std::fs::read_to_string(path).unwrap(), FILE);
}

#[test]
fn revert_drops_a_captured_combination() {
    let (mut h, _) = window_on("keys-revert");
    click(&mut h, Role::Button, "Change Focus down");
    press(&mut h, Modifiers::CTRL, Key::H);
    h.get_by_label("1 unsaved change");

    click(&mut h, Role::Button, "Revert");

    h.get_by_label("No unsaved changes");
    assert!(h.query_by_label("H").is_none());
}

#[test]
fn reopening_shows_the_saved_keys_and_search_finds_them() {
    // Arrange
    let (mut h, path) = window_on("keys-reopen");
    click(&mut h, Role::Button, "Change Focus down");
    press(&mut h, Modifiers::ALT, Key::Y);
    click(&mut h, Role::Button, "Apply");

    // Act
    let dir = path.parent().unwrap().to_path_buf();
    let mut again = window(
        ConfigStore::new(dir),
        DaemonState::Stopped,
        &SignIn::default(),
    );
    open(&mut again, "Keybindings");
    search(&mut again, "alt+y");
    again.run();

    // Assert
    again.get_by_label("Y");
    again.get_by_role_and_label(Role::Button, "Change Focus down");
    assert!(
        again
            .query_by_role_and_label(Role::Button, "Change Focus up")
            .is_none()
    );
}
