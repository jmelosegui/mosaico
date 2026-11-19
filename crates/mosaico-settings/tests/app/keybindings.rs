//! The Keybindings page.

use eframe::egui::{Key, Modifiers};

use crate::support::*;

pub const FILE: &str = "# my keys\n\n[[keybinding]]\naction = \"focus-down\" # down\nkey = \"J\"\nmodifiers = [\"alt\"]\n\n[[keybinding]]\naction = \"focus-up\"\nkey = \"K\"\nmodifiers = [\"alt\"]\n";

/// What an action row says while it waits for a key press.
pub const PROMPT: &str = "Press the new shortcut, or Backspace for none";

pub fn window_on(test: &str) -> (Harness<'static, SettingsApp>, PathBuf) {
    let store = folder(test);
    let path = store.keybindings_path();
    std::fs::write(&path, FILE).unwrap();
    let mut h = window(store, DaemonState::Stopped, &SignIn::default());
    open(&mut h, "Keybindings");
    (h, path)
}

pub fn filter(h: &mut Harness<'static, SettingsApp>, text: &str) {
    h.get_by_role_and_label(Role::TextInput, "Filter keybindings")
        .click();
    h.run();
    h.get_by_role_and_label(Role::TextInput, "Filter keybindings")
        .type_text(text);
    h.run();
}

pub fn press(h: &mut Harness<'static, SettingsApp>, modifiers: Modifiers, key: Key) {
    h.key_press_modifiers(modifiers, key);
    h.run();
}

#[test]
fn a_captured_combination_is_saved_and_keeps_the_comments() {
    // Arrange
    let (mut h, path) = window_on("keys-capture");
    click(&mut h, Role::Button, "Change Focus down");
    h.get_by_label(PROMPT);

    // Act
    press(&mut h, Modifiers::ALT | Modifiers::SHIFT, Key::Y);

    // Assert
    h.get_by_label("1 unsaved change");
    assert!(h.query_by_label(PROMPT).is_none());
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

    assert!(h.query_by_label(PROMPT).is_none());
    h.get_by_label("No unsaved changes");
    assert_eq!(std::fs::read_to_string(path).unwrap(), FILE);
}

#[test]
fn backspace_leaves_the_action_on_no_key_and_it_can_be_set_again() {
    // Arrange
    let (mut h, path) = window_on("keys-clear");
    click(&mut h, Role::Button, "Change Focus down");

    // Act
    press(&mut h, Modifiers::NONE, Key::Backspace);
    click(&mut h, Role::Button, "Apply");

    // Assert -- the action stays in the file, so the daemon does not
    // give it its default key back.
    assert!(h.query_by_label(PROMPT).is_none());
    let saved = std::fs::read_to_string(&path).unwrap();
    assert!(saved.contains("action = \"focus-down\" # down"), "{saved}");
    assert!(!saved.contains("key = \"J\""), "{saved}");
    h.get_by_label("Changes saved");

    // Act -- set a key on it again.
    click(&mut h, Role::Button, "Set Focus down");
    press(&mut h, Modifiers::ALT, Key::Y);
    click(&mut h, Role::Button, "Apply");

    // Assert
    let saved = std::fs::read_to_string(&path).unwrap();
    assert!(saved.contains("key = \"Y\""), "{saved}");
}

#[test]
fn a_combination_bound_twice_in_the_file_is_flagged_and_not_saved() {
    // Arrange
    let store = folder("keys-conflict");
    let path = store.keybindings_path();
    let twice = FILE.replace("\"K\"", "\"J\"")
        + "\n[[keybinding]]\naction = \"focus-left\"\nkey = \"H\"\nmodifiers = [\"alt\"]\n";
    std::fs::write(&path, &twice).unwrap();
    let mut h = window(store, DaemonState::Stopped, &SignIn::default());
    open(&mut h, "Keybindings");

    // Act
    click(&mut h, Role::Button, "Change Focus left");
    press(&mut h, Modifiers::ALT, Key::Y);
    click(&mut h, Role::Button, "Apply");

    // Assert
    assert_eq!(h.query_all_by_label("Conflict").count(), 2);
    h.get_by_label_contains("Also bound to Focus down.");
    h.get_by_label_contains("Also bound to Focus up.");
    assert!(matches!(h.state().applied(), Some(Applied::Failed(_))));
    assert_eq!(std::fs::read_to_string(path).unwrap(), twice);
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
    filter(&mut again, "alt+y");

    // Assert
    again.get_by_label("Y");
    again.get_by_role_and_label(Role::Button, "Change Focus down");
    assert!(
        again
            .query_by_role_and_label(Role::Button, "Change Focus up")
            .is_none()
    );
}

#[test]
fn backspace_does_not_leave_an_application_on_no_key() {
    // Arrange
    let store = folder("keys-clear-app");
    let path = store.keybindings_path();
    let file =
        format!("{FILE}\n[[launch]]\ncommand = \"notepad\"\nkey = \"N\"\nmodifiers = [\"alt\"]\n");
    std::fs::write(&path, &file).unwrap();
    let mut h = window(store, DaemonState::Stopped, &SignIn::default());
    open(&mut h, "Keybindings");
    click(&mut h, Role::Tab, "Applications");
    click(&mut h, Role::Button, "Change shortcut of notepad");

    // Act
    press(&mut h, Modifiers::NONE, Key::Backspace);

    // Assert
    h.get_by_label("An application needs a shortcut.");
    h.get_by_label("No unsaved changes");
    assert_eq!(std::fs::read_to_string(path).unwrap(), file);
}

#[test]
fn an_application_can_be_set_to_start_without_a_window() {
    // Arrange
    let store = folder("keys-app-hidden");
    let path = store.keybindings_path();
    let file =
        format!("{FILE}\n[[launch]]\ncommand = \"pwsh\"\nkey = \"N\"\nmodifiers = [\"alt\"]\n");
    std::fs::write(&path, &file).unwrap();
    let mut h = window(store, DaemonState::Stopped, &SignIn::default());
    open(&mut h, "Keybindings");
    click(&mut h, Role::Tab, "Applications");
    click(&mut h, Role::Button, "Edit pwsh");

    // Act
    click(&mut h, Role::CheckBox, "Start it without a window");
    click(&mut h, Role::Button, "Apply");

    // Assert
    let saved = std::fs::read_to_string(path).unwrap();
    assert!(
        saved.ends_with("modifiers = [\"alt\"]\nhidden = true\n"),
        "{saved}"
    );
}
