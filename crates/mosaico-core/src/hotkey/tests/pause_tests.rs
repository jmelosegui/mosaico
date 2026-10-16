use crate::hotkey::Hotkeys;
use crate::{Action, Direction};

use super::fake_registrar::{Call, FakeRegistrar, alt_j, alt_k, alt_shift, bind, bindings};

#[test]
fn pausing_keeps_the_toggle_binding_registered() {
    // Arrange
    let registrar = FakeRegistrar::default();
    let (mut hotkeys, _) = Hotkeys::register(registrar.clone(), &bindings());

    // Act
    let rejected = hotkeys.pause();

    // Assert
    assert!(rejected.is_empty());
    assert!(hotkeys.is_paused());
    assert_eq!(registrar.held(), vec![alt_shift('P')]);
}

#[test]
fn pausing_without_a_toggle_binding_gives_back_every_hotkey() {
    // Arrange
    let registrar = FakeRegistrar::default();
    let mut bindings = bindings();
    bindings.retain(|binding| binding.action != Action::TogglePause);
    let (mut hotkeys, _) = Hotkeys::register(registrar.clone(), &bindings);

    // Act
    hotkeys.pause();

    // Assert
    assert!(registrar.held().is_empty());
}

#[test]
fn every_toggle_binding_stays_registered_while_paused() {
    // Arrange
    let registrar = FakeRegistrar::default();
    let mut bindings = bindings();
    bindings.push(bind(Action::TogglePause, "F12", &[]));
    let (mut hotkeys, _) = Hotkeys::register(registrar.clone(), &bindings);
    let second_toggle = *registrar.held().last().expect("the F12 binding");

    // Act
    hotkeys.pause();

    // Assert
    assert_eq!(registrar.held(), vec![alt_shift('P'), second_toggle]);
}

#[test]
fn while_paused_a_hotkey_the_pause_gave_back_does_not_trigger() {
    // Arrange
    let registrar = FakeRegistrar::default();
    let (mut hotkeys, _) = Hotkeys::register(registrar.clone(), &bindings());
    let focus = registrar.id_of(alt_j());
    let toggle = registrar.id_of(alt_shift('P'));

    // Act
    hotkeys.pause();

    // Assert
    assert_eq!(hotkeys.triggered(focus), None);
    assert_eq!(hotkeys.triggered(toggle), Some(Action::TogglePause));
}

#[test]
fn unpause_restores_exactly_the_paused_set() {
    // Arrange -- Alt+K is taken at the start, so it was never registered.
    let registrar = FakeRegistrar::default();
    registrar.desktop().taken.insert(alt_k());
    let (mut hotkeys, _) = Hotkeys::register(registrar.clone(), &bindings());
    let focus = registrar.id_of(alt_j());
    let retile = registrar.id_of(alt_shift('R'));
    hotkeys.pause();
    registrar.take_calls();

    // Act
    let rejected = hotkeys.unpause();

    // Assert
    assert!(rejected.is_empty());
    assert!(!hotkeys.is_paused());
    let calls = vec![Call::Register(focus), Call::Register(retile)];
    assert_eq!(registrar.take_calls(), calls);
    let expected = vec![alt_j(), alt_shift('R'), alt_shift('P')];
    assert_eq!(registrar.held(), expected);
}

#[test]
fn pausing_twice_gives_the_hotkeys_back_once() {
    // Arrange
    let registrar = FakeRegistrar::default();
    let (mut hotkeys, _) = Hotkeys::register(registrar.clone(), &bindings());
    hotkeys.pause();
    registrar.take_calls();

    // Act
    let rejected = hotkeys.pause();

    // Assert
    assert!(rejected.is_empty());
    assert_eq!(registrar.take_calls(), vec![]);
}

#[test]
fn unpausing_hotkeys_that_are_not_paused_does_nothing() {
    // Arrange
    let registrar = FakeRegistrar::default();
    let (mut hotkeys, _) = Hotkeys::register(registrar.clone(), &bindings());
    registrar.take_calls();

    // Act
    let rejected = hotkeys.unpause();

    // Assert
    assert!(rejected.is_empty());
    assert_eq!(registrar.take_calls(), vec![]);
}

#[test]
fn a_hotkey_taken_while_paused_is_reported_and_tried_again_at_the_next_unpause() {
    // Arrange
    let registrar = FakeRegistrar::default();
    let (mut hotkeys, _) = Hotkeys::register(registrar.clone(), &bindings());
    hotkeys.pause();
    registrar.desktop().taken.insert(alt_j());

    // Act
    let rejected = hotkeys.unpause();
    registrar.desktop().taken.clear();
    hotkeys.pause();
    let rejected_again = hotkeys.unpause();

    // Assert
    assert_eq!(rejected.len(), 1);
    assert_eq!(rejected[0].combo, Some(alt_j()));
    assert!(rejected_again.is_empty());
    let expected = vec![alt_j(), alt_k(), alt_shift('R'), alt_shift('P')];
    assert_eq!(registrar.held(), expected);
}

#[test]
fn a_hotkey_the_system_keeps_at_a_pause_is_reported_and_still_runs() {
    // Arrange
    let registrar = FakeRegistrar::default();
    registrar.desktop().stuck.insert(alt_j());
    let (mut hotkeys, _) = Hotkeys::register(registrar.clone(), &bindings());
    let focus = registrar.id_of(alt_j());

    // Act
    let rejected = hotkeys.pause();
    let triggered_while_paused = hotkeys.triggered(focus);

    // Assert -- its keystrokes still do not reach the applications, so
    // dropping its presses would leave the keys doing nothing at all.
    assert_eq!(rejected.len(), 1);
    assert_eq!(rejected[0].combo, Some(alt_j()));
    assert_eq!(registrar.held(), vec![alt_j(), alt_shift('P')]);
    assert_eq!(triggered_while_paused, Some(Action::Focus(Direction::Down)));
}

#[test]
fn a_hotkey_the_system_kept_at_the_pause_is_not_registered_twice() {
    // Arrange
    let registrar = FakeRegistrar::default();
    registrar.desktop().stuck.insert(alt_j());
    let (mut hotkeys, _) = Hotkeys::register(registrar.clone(), &bindings());
    let focus = registrar.id_of(alt_j());
    hotkeys.pause();
    registrar.take_calls();

    // Act
    let rejected = hotkeys.unpause();

    // Assert
    assert!(rejected.is_empty());
    assert!(!registrar.take_calls().contains(&Call::Register(focus)));
}

#[test]
fn dropping_the_hotkeys_gives_back_exactly_what_is_still_registered() {
    // Arrange
    let registrar = FakeRegistrar::default();
    let (mut hotkeys, _) = Hotkeys::register(registrar.clone(), &bindings());
    let toggle = registrar.id_of(alt_shift('P'));
    hotkeys.pause();
    registrar.take_calls();

    // Act
    drop(hotkeys);

    // Assert
    assert_eq!(registrar.take_calls(), vec![Call::Unregister(toggle)]);
    assert!(registrar.held().is_empty());
}
