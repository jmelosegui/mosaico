//! Launch bindings registered beside the keybindings.

use crate::config::{LaunchBinding, Modifier, WindowMatch};
use crate::hotkey::{Hotkeys, Target};

use super::fake_registrar::{FakeRegistrar, alt_j, alt_shift, bindings};

fn launch(command: &str, key: &str) -> LaunchBinding {
    LaunchBinding {
        command: command.into(),
        args: Vec::new(),
        key: key.into(),
        modifiers: vec![Modifier::Alt, Modifier::Shift],
        focus: false,
        windows: WindowMatch::default(),
    }
}

#[test]
fn a_press_of_a_launch_key_names_its_launch_binding() {
    // Arrange
    let registrar = FakeRegistrar::default();
    let launches = [launch("wt.exe", "T"), launch("msedge.exe", "B")];
    let (hotkeys, rejected) = Hotkeys::register(registrar.clone(), &bindings(), &launches);

    // Act
    let browser = hotkeys.triggered(registrar.id_of(alt_shift('B')));
    let terminal = hotkeys.triggered(registrar.id_of(alt_shift('T')));

    // Assert
    assert!(rejected.is_empty(), "{rejected:?}");
    assert_eq!(browser, Some(Target::Launch(1)));
    assert_eq!(terminal, Some(Target::Launch(0)));
}

#[test]
fn a_launch_key_bound_to_an_action_already_is_rejected_by_its_command() {
    // Arrange -- Alt+Shift+R is retile in the keybindings.
    let registrar = FakeRegistrar::default();
    let launches = [launch("wt.exe", "R")];

    // Act
    let (hotkeys, rejected) = Hotkeys::register(registrar.clone(), &bindings(), &launches);

    // Assert -- the keybinding keeps the combination.
    let shown: Vec<String> = rejected.iter().map(ToString::to_string).collect();
    assert_eq!(
        shown,
        ["Alt+Shift+R (launch wt.exe): could not register a hotkey: it is taken"]
    );
    let retile = hotkeys.triggered(registrar.id_of(alt_shift('R')));
    assert_ne!(retile, Some(Target::Launch(0)));
}

#[test]
fn pausing_gives_the_launch_keys_back_and_unpausing_takes_them_again() {
    // Arrange
    let registrar = FakeRegistrar::default();
    let launches = [launch("wt.exe", "T")];
    let (mut hotkeys, _) = Hotkeys::register(registrar.clone(), &bindings(), &launches);
    let id = registrar.id_of(alt_shift('T'));

    // Act
    hotkeys.pause();
    let while_paused = hotkeys.triggered(id);
    let held_while_paused = registrar.held();
    hotkeys.unpause();

    // Assert
    assert_eq!(while_paused, None);
    assert!(!held_while_paused.contains(&alt_shift('T')));
    assert!(!held_while_paused.contains(&alt_j()));
    assert_eq!(hotkeys.triggered(id), Some(Target::Launch(0)));
}
