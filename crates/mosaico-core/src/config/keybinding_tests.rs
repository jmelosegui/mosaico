use crate::Error;
use crate::hotkey::{KeyCombo, Modifiers, letter};

use super::*;

#[test]
fn a_keybinding_is_triggered_by_the_combo_it_names() {
    // Arrange
    let binding = bind(Action::Retile, "r", &[Modifier::Alt, Modifier::Shift]);

    // Act
    let combo = binding.combo().ok();

    // Assert
    let expected = KeyCombo {
        modifiers: Modifiers::ALT | Modifiers::SHIFT,
        key: letter('R'),
    };
    assert_eq!(combo, Some(expected));
}

#[test]
fn a_keybinding_with_an_unknown_key_name_has_no_combo() {
    // Arrange
    let binding = bind(Action::Retile, "Foo", &[Modifier::Alt]);

    // Act
    let result = binding.combo();

    // Assert
    assert!(matches!(result, Err(Error::Config { .. })));
}

#[test]
fn every_default_keybinding_has_a_combo() {
    // Act
    let bindings = defaults();

    // Assert
    for binding in &bindings {
        assert!(
            binding.combo().is_ok(),
            "{} {}",
            binding.action,
            binding.key
        );
    }
}
