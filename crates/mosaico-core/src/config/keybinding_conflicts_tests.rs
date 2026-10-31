use crate::Action;
use crate::action::Direction;
use crate::config::Modifier::{self, Alt, Ctrl, Shift};
use crate::config::keybinding;

use super::*;

fn bind(action: Action, key: &str, modifiers: &[Modifier]) -> Keybinding {
    Keybinding {
        action,
        key: key.into(),
        modifiers: modifiers.to_vec(),
    }
}

#[test]
fn the_defaults_have_no_conflict() {
    assert!(conflicts(&keybinding::defaults()).is_empty());
}

#[test]
fn modifier_order_and_key_case_do_not_hide_a_conflict() {
    // Arrange
    let bindings = [
        bind(Action::Focus(Direction::Down), "J", &[Alt, Shift]),
        bind(Action::Retile, "R", &[Alt]),
        bind(Action::Move(Direction::Down), "j", &[Shift, Alt]),
        bind(Action::CloseFocused, "J", &[Shift, Alt, Alt]),
    ];

    // Act
    let found = conflicts(&bindings);

    // Assert
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].bindings, [0, 2, 3]);
    assert_eq!(found[0].combo.to_string(), "Alt+Shift+J");
}

#[test]
fn the_same_key_with_other_modifiers_is_not_a_conflict() {
    let bindings = [
        bind(Action::Retile, "J", &[Alt]),
        bind(Action::CycleLayout, "J", &[Ctrl]),
        bind(Action::TogglePause, "J", &[]),
    ];

    assert!(conflicts(&bindings).is_empty());
}

#[test]
fn names_of_one_key_clash_and_unknown_keys_clash_with_nothing() {
    let bindings = [
        bind(Action::Retile, "Esc", &[Alt]),
        bind(Action::CycleLayout, "escape", &[Alt]),
        bind(Action::TogglePause, "nope", &[Alt]),
        bind(Action::ToggleFloat, "nope", &[Alt]),
    ];

    let found = conflicts(&bindings);

    assert_eq!(found.len(), 1);
    assert_eq!(found[0].bindings, [0, 1]);
}
