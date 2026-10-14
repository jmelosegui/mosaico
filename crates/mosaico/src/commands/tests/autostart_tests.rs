use mosaico_core::system::LaunchCommand;

use super::*;
use crate::commands::test_support::{EXE, Harness};

#[test]
fn enabling_registers_and_says_so() {
    // Arrange
    let mut h = Harness::new("autostart-enable");

    // Act
    let exit = h.run(enable);

    // Assert
    assert_eq!(exit, Exit::Success);
    assert_eq!(
        *h.world.autostart.borrow(),
        Some(LaunchCommand::autostart(EXE)),
        "the executable the command runs from is registered to start"
    );
    assert_eq!(h.out(), "Autostart enabled.\n");
}

#[test]
fn a_registration_error_fails_the_command() {
    // Arrange
    let mut h = Harness::new("autostart-error");
    *h.world.autostart_error.borrow_mut() = Some("access denied".into());

    // Act
    let exit = h.run(disable);

    // Assert
    assert_eq!(exit, Exit::Failure);
    assert_eq!(h.err(), "Error: access denied\n");
}

#[test]
fn status_reports_the_registration() {
    // Arrange
    let mut h = Harness::new("autostart-status");
    *h.world.autostart.borrow_mut() = Some(LaunchCommand::autostart(EXE));

    // Act
    let exit = h.run(status);

    // Assert
    assert_eq!(exit, Exit::Success);
    assert_eq!(h.out(), "Autostart is currently enabled.\n");
}

#[test]
fn a_registration_that_cannot_be_read_fails_the_status() {
    // Arrange
    let mut h = Harness::new("autostart-status-unreadable");
    *h.world.autostart_error.borrow_mut() = Some("access denied".into());

    // Act
    let exit = h.run(status);

    // Assert
    assert_eq!(exit, Exit::Failure);
    assert_eq!(h.err(), "Error: access denied\n");
    assert_eq!(h.out(), "", "not reported as disabled");
}
