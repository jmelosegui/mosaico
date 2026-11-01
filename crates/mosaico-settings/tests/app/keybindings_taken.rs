//! The Keybindings page: filtering, restoring defaults and shortcuts that are refused or taken.

use eframe::egui::{Key, Modifiers};

use crate::keybindings::{FILE, PROMPT, filter, press, window_on};
use crate::support::*;

#[test]
fn the_filter_keeps_the_bindings_matching_an_action_or_a_key() {
    let (mut h, _) = window_on("keys-filter");

    filter(&mut h, "focus-up");

    h.get_by_role_and_label(Role::Button, "Change Focus up");
    assert!(
        h.query_by_role_and_label(Role::Button, "Change Focus down")
            .is_none()
    );
}

#[test]
fn restore_defaults_puts_back_the_keys_and_the_bindings_the_file_lost() {
    // Arrange
    let (mut h, path) = window_on("keys-restore");
    click(&mut h, Role::Button, "Change Focus down");
    press(&mut h, Modifiers::ALT, Key::Y);

    // Act
    click(&mut h, Role::Button, "Restore defaults");
    h.get_by_role_and_label(Role::Button, "Change Move left");
    click(&mut h, Role::Button, "Apply");

    // Assert
    let saved = std::fs::read_to_string(path).unwrap();
    assert!(!saved.contains("\"Y\""));
    assert!(saved.contains("action = \"move-left\""));
    h.get_by_label("Changes saved");
}

#[test]
fn a_taken_combination_asks_before_it_is_used() {
    // Arrange
    let (mut h, path) = window_on("keys-taken");
    click(&mut h, Role::Button, "Change Focus up");

    // Act
    press(&mut h, Modifiers::ALT, Key::J);

    // Assert
    h.get_by_label("Alt J is already bound to");
    assert_eq!(h.query_all_by_label("Focus down").count(), 2);
    h.get_by_label("No unsaved changes");
    assert_eq!(std::fs::read_to_string(path).unwrap(), FILE);
}

#[test]
fn cancelling_a_taken_combination_leaves_the_bindings_alone() {
    let (mut h, _) = window_on("keys-taken-cancel");
    click(&mut h, Role::Button, "Change Focus up");
    press(&mut h, Modifiers::ALT, Key::J);

    click(&mut h, Role::Button, "Cancel");

    assert!(h.query_by_label("Alt J is already bound to").is_none());
    assert!(h.query_by_label(PROMPT).is_none());
    h.get_by_label("No unsaved changes");
    h.get_by_role_and_label(Role::Button, "Change Focus down");
}

#[test]
fn replacing_a_taken_combination_takes_it_from_the_other_binding() {
    // Arrange
    let (mut h, path) = window_on("keys-taken-replace");
    click(&mut h, Role::Button, "Change Focus up");
    press(&mut h, Modifiers::ALT, Key::J);

    // Act
    click(&mut h, Role::Button, "Replace");
    click(&mut h, Role::Button, "Apply");

    // Assert
    assert!(
        h.query_by_role_and_label(Role::Button, "Change Focus down")
            .is_none()
    );
    h.get_by_role_and_label(Role::Button, "Change Focus up");
    let saved = std::fs::read_to_string(path).unwrap();
    assert!(!saved.contains("focus-down"));
    assert!(saved.contains("action = \"focus-up\""));
    h.get_by_label("Changes saved");
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
