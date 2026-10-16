//! Hotkey tests against a registrar that keeps its hotkeys in memory.

use crate::config::Modifier;
use crate::hotkey::{HotkeyId, Hotkeys};
use crate::{Action, Direction, Error};

use fake_registrar::{Call, FakeRegistrar, alt_j, alt_k, alt_shift, bind, bindings};

#[path = "fake_registrar.rs"]
mod fake_registrar;
#[path = "pause_tests.rs"]
mod pause_tests;

#[test]
fn every_binding_is_registered_with_its_combination() {
    // Arrange
    let registrar = FakeRegistrar::default();

    // Act
    let (_hotkeys, rejected) = Hotkeys::register(registrar.clone(), &bindings());

    // Assert
    assert!(rejected.is_empty());
    let expected = vec![alt_j(), alt_k(), alt_shift('R'), alt_shift('P')];
    assert_eq!(registrar.held(), expected);
}

#[test]
fn an_unknown_key_name_leaves_out_only_its_own_binding() {
    // Arrange
    let registrar = FakeRegistrar::default();
    let mut bindings = bindings();
    bindings.insert(1, bind(Action::ToggleMonocle, "Foo", &[Modifier::Alt]));

    // Act
    let (_hotkeys, rejected) = Hotkeys::register(registrar.clone(), &bindings);

    // Assert
    assert_eq!(rejected.len(), 1);
    assert_eq!(rejected[0].action, Action::ToggleMonocle);
    assert_eq!(rejected[0].combo, None);
    assert!(matches!(rejected[0].error, Error::Config { .. }));
    let expected = vec![alt_j(), alt_k(), alt_shift('R'), alt_shift('P')];
    assert_eq!(registrar.held(), expected);
}

#[test]
fn a_combination_another_program_holds_is_reported_and_the_rest_still_work() {
    // Arrange
    let registrar = FakeRegistrar::default();
    registrar.desktop().taken.insert(alt_k());

    // Act
    let (_hotkeys, rejected) = Hotkeys::register(registrar.clone(), &bindings());

    // Assert
    assert_eq!(rejected.len(), 1);
    assert_eq!(rejected[0].action, Action::Focus(Direction::Up));
    assert_eq!(rejected[0].combo, Some(alt_k()));
    assert!(matches!(rejected[0].error, Error::Platform { .. }));
    let expected = vec![alt_j(), alt_shift('R'), alt_shift('P')];
    assert_eq!(registrar.held(), expected);
}

#[test]
fn a_press_runs_the_action_of_its_binding() {
    // Arrange
    let registrar = FakeRegistrar::default();
    let (hotkeys, _) = Hotkeys::register(registrar.clone(), &bindings());
    let id = registrar.id_of(alt_shift('R'));

    // Act
    let action = hotkeys.triggered(id);

    // Assert
    assert_eq!(action, Some(Action::Retile));
}

#[test]
fn a_press_with_an_id_that_was_never_registered_does_nothing() {
    // Arrange
    let registrar = FakeRegistrar::default();
    registrar.desktop().taken.insert(alt_j());
    let (hotkeys, _) = Hotkeys::register(registrar.clone(), &bindings());
    let Call::Register(refused) = registrar.take_calls()[0] else {
        panic!("the first call registers the first binding");
    };

    // Act
    let refused_action = hotkeys.triggered(refused);
    let unknown_action = hotkeys.triggered(HotkeyId::from_raw(999));

    // Assert
    assert_eq!(refused_action, None);
    assert_eq!(unknown_action, None);
}

#[test]
fn a_rejected_binding_is_shown_with_its_keys_action_and_reason() {
    // Arrange
    let registrar = FakeRegistrar::default();
    registrar.desktop().taken.insert(alt_j());
    let mut bindings = bindings();
    bindings.push(bind(Action::ToggleMonocle, "Foo", &[Modifier::Alt]));

    // Act
    let (_hotkeys, rejected) = Hotkeys::register(registrar, &bindings);

    // Assert
    let shown: Vec<String> = rejected.iter().map(ToString::to_string).collect();
    assert_eq!(
        shown[0],
        "Alt+J (focus-down): could not register a hotkey: it is taken"
    );
    assert!(
        shown[1].starts_with("toggle-monocle: unknown key name \"Foo\""),
        "{}",
        shown[1]
    );
}
