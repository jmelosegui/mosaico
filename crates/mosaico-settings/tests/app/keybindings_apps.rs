//! The Applications tab of the Keybindings page: shortcuts that start a
//! program.

use eframe::egui::{Key, Modifiers};

use crate::keybindings::{FILE, press};
use crate::support::*;

const NOTEPAD: &str =
    "\n# apps\n[[launch]]\ncommand = \"notepad\" # editor\nkey = \"N\"\nmodifiers = [\"alt\"]\n";

fn window_with(test: &str, file: &str) -> (Harness<'static, SettingsApp>, PathBuf) {
    let store = folder(test);
    let path = store.keybindings_path();
    std::fs::write(&path, file).unwrap();
    let mut h = window(store, DaemonState::Stopped, &SignIn::default());
    open(&mut h, "Keybindings");
    click(&mut h, Role::Tab, "Applications");
    (h, path)
}

fn type_into(h: &mut Harness<'static, SettingsApp>, label: &str, text: &str) {
    h.get_by_role_and_label(Role::TextInput, label).click();
    h.run();
    h.get_by_role_and_label(Role::TextInput, label)
        .type_text(text);
    h.run();
}

#[test]
fn an_added_application_is_saved_with_its_shortcut() {
    // Arrange
    let (mut h, path) = window_with("apps-add", FILE);
    click(&mut h, Role::Button, "Add application");
    type_into(&mut h, "Command of application 1", "notepad");

    // Act
    click(&mut h, Role::Button, "Change shortcut of notepad");
    press(&mut h, Modifiers::ALT, Key::T);
    click(&mut h, Role::Button, "Apply");

    // Assert
    let saved = std::fs::read_to_string(&path).unwrap();
    assert!(saved.starts_with(FILE), "{saved}");
    assert!(
        saved.contains("[[launch]]\ncommand = \"notepad\"\n"),
        "{saved}"
    );
    assert!(
        saved.contains("key = \"T\"\nmodifiers = [\"alt\"]"),
        "{saved}"
    );
    assert!(!saved.contains("focus ="), "{saved}");
    h.get_by_label("Changes saved");
}

#[test]
fn a_new_application_records_its_shortcut_from_its_editor() {
    // Arrange
    let (mut h, path) = window_with("apps-record", FILE);
    click(&mut h, Role::Button, "Add application");
    type_into(&mut h, "Command of application 1", "notepad");

    // Act
    click(&mut h, Role::Button, "Record the shortcut of application 1");
    press(&mut h, Modifiers::ALT, Key::T);
    click(&mut h, Role::Button, "Apply");

    // Assert
    let saved = std::fs::read_to_string(&path).unwrap();
    assert!(
        saved.contains("command = \"notepad\"\nkey = \"T\"\nmodifiers = [\"alt\"]"),
        "{saved}"
    );
}

#[test]
fn an_application_without_a_shortcut_offers_to_set_one_on_its_row() {
    // Arrange
    let (mut h, path) = window_with("apps-set", FILE);
    click(&mut h, Role::Button, "Add application");
    type_into(&mut h, "Command of application 1", "notepad");

    // Act
    click(&mut h, Role::Button, "Set shortcut of notepad");
    press(&mut h, Modifiers::ALT, Key::T);
    click(&mut h, Role::Button, "Apply");

    // Assert
    let saved = std::fs::read_to_string(&path).unwrap();
    assert!(
        saved.contains("command = \"notepad\"\nkey = \"T\"\nmodifiers = [\"alt\"]"),
        "{saved}"
    );
    assert!(
        h.query_by_role_and_label(Role::Button, "Set shortcut of notepad")
            .is_none()
    );
}

#[test]
fn an_application_without_a_shortcut_is_not_saved() {
    // Arrange
    let (mut h, path) = window_with("apps-no-key", FILE);
    click(&mut h, Role::Button, "Add application");
    type_into(&mut h, "Command of application 1", "notepad");

    // Act
    click(&mut h, Role::Button, "Apply");

    // Assert
    h.get_by_label_contains("launch binding 1 (notepad) has no shortcut");
    assert_eq!(std::fs::read_to_string(path).unwrap(), FILE);
}

#[test]
fn editing_an_application_keeps_the_comments_of_the_file() {
    // Arrange
    let file = format!("{FILE}{NOTEPAD}");
    let (mut h, path) = window_with("apps-edit", &file);
    click(&mut h, Role::Button, "Edit notepad");

    // Act
    type_into(&mut h, "Arguments of application 1", "\"my notes.txt\"");
    click(
        &mut h,
        Role::CheckBox,
        "Focus an open window of it instead of starting another",
    );
    click(&mut h, Role::Button, "Done");
    click(&mut h, Role::Button, "Apply");

    // Assert
    let saved = std::fs::read_to_string(&path).unwrap();
    assert!(saved.starts_with(FILE), "{saved}");
    assert!(
        saved.contains("# apps\n[[launch]]\ncommand = \"notepad\" # editor\n"),
        "{saved}"
    );
    assert!(saved.contains("args = [\"my notes.txt\"]"), "{saved}");
    assert!(saved.contains("focus = true"), "{saved}");
    assert!(
        h.query_by_role_and_label(Role::TextInput, "Arguments of application 1")
            .is_none()
    );
}

#[test]
fn the_path_of_an_application_s_windows_is_asked_only_when_it_focuses_one() {
    // Arrange
    let file = format!("{FILE}{NOTEPAD}");
    let (mut h, path) = window_with("apps-path", &file);
    click(&mut h, Role::Button, "Edit notepad");
    let field = "Its windows' path ends with of application 1";
    assert!(h.query_by_role_and_label(Role::TextInput, field).is_none());

    // Act
    click(
        &mut h,
        Role::CheckBox,
        "Focus an open window of it instead of starting another",
    );
    type_into(&mut h, field, "notepad2.exe");
    click(&mut h, Role::Button, "Apply");

    // Assert
    let saved = std::fs::read_to_string(&path).unwrap();
    assert!(saved.contains("focus = true"), "{saved}");
    assert!(saved.contains("match_path = \"notepad2.exe\""), "{saved}");
}

#[test]
fn a_removed_application_leaves_the_file() {
    // Arrange
    let file = format!("{FILE}{NOTEPAD}");
    let (mut h, path) = window_with("apps-remove", &file);

    // Act
    click(&mut h, Role::Button, "Remove notepad");
    click(&mut h, Role::Button, "Apply");

    // Assert
    let saved = std::fs::read_to_string(&path).unwrap();
    assert!(!saved.contains("[[launch]]"), "{saved}");
    assert!(saved.starts_with(FILE.trim_end()), "{saved}");
    h.get_by_label("No applications yet. Add one to start it with a shortcut.");
}

#[test]
fn an_application_sharing_a_shortcut_with_an_action_is_flagged_and_not_saved() {
    // Arrange
    let file = format!("{FILE}{}", NOTEPAD.replace("\"N\"", "\"J\""));
    let (mut h, path) = window_with("apps-clash", &file);
    click(&mut h, Role::Tab, "Actions");

    // Act
    click(&mut h, Role::Button, "Change Focus up");
    press(&mut h, Modifiers::ALT, Key::Y);
    click(&mut h, Role::Button, "Apply");

    // Assert
    h.get_by_label_contains("Also bound to notepad.");
    assert!(matches!(h.state().applied(), Some(Applied::Failed(_))));
    assert_eq!(std::fs::read_to_string(path).unwrap(), file);
    click(&mut h, Role::Tab, "Applications");
    h.get_by_label_contains("Also bound to Focus down.");
}

#[test]
fn the_tabs_keep_actions_and_applications_apart() {
    // Arrange
    let file = format!("{FILE}{NOTEPAD}");
    let (mut h, _) = window_with("apps-tabs", &file);
    h.get_by_role_and_label(Role::Button, "Edit notepad");
    assert!(
        h.query_by_role_and_label(Role::Button, "Change Focus up")
            .is_none()
    );

    // Act
    click(&mut h, Role::Tab, "Actions");

    // Assert
    h.get_by_role_and_label(Role::Button, "Change Focus up");
    h.get_by_role_and_label(Role::TextInput, "Filter keybindings");
    assert!(
        h.query_by_role_and_label(Role::Button, "Edit notepad")
            .is_none()
    );
}

#[test]
fn a_search_finds_an_application_by_its_command() {
    // Arrange
    let file = format!("{FILE}{NOTEPAD}");
    let (mut h, _) = window_with("apps-search", &file);
    click(&mut h, Role::Tab, "Actions");

    // Act
    search(&mut h, "notep");
    h.run();

    // Assert
    h.get_by_role_and_label(Role::Button, "Edit notepad");
    assert!(
        h.query_by_role_and_label(Role::Button, "Change Focus up")
            .is_none()
    );
}

#[test]
fn a_key_pressed_on_the_actions_tab_does_not_rebind_a_waiting_application() {
    // Arrange
    let file = format!("{FILE}{NOTEPAD}");
    let (mut h, _) = window_with("apps-hidden-capture", &file);
    click(&mut h, Role::Button, "Change shortcut of notepad");
    click(&mut h, Role::Tab, "Actions");

    // Act
    press(&mut h, Modifiers::ALT, Key::T);

    // Assert
    h.get_by_label("No unsaved changes");
    click(&mut h, Role::Tab, "Applications");
    h.get_by_label("Press the new shortcut");
}

#[test]
fn an_open_editor_stays_open_and_follows_the_file_while_nothing_is_edited() {
    // Arrange
    let file = format!("{FILE}{NOTEPAD}");
    let (mut h, path) = window_with("apps-follow", &file);
    click(&mut h, Role::Button, "Edit notepad");

    // Act
    std::fs::write(&path, file.replace("\"notepad\"", "\"wordpad\"")).unwrap();
    for _ in 0..20 {
        h.step();
    }
    h.run();

    // Assert
    h.get_by_role_and_label(Role::Button, "Edit wordpad");
    h.get_by_role_and_label(Role::TextInput, "Command of application 1");
    h.get_by_label("No unsaved changes");
}
