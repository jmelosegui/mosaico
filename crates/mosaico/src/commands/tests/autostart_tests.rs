use super::*;
use crate::commands::test_support::Harness;

#[test]
fn enabling_registers_and_says_so() {
    // Arrange
    let mut h = Harness::new("autostart-enable");

    // Act
    let exit = h.run(enable);

    // Assert
    assert_eq!(exit, Exit::Success);
    assert!(h.world.autostart.get());
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
    h.world.autostart.set(true);

    // Act
    let exit = h.run(status);

    // Assert
    assert_eq!(exit, Exit::Success);
    assert_eq!(h.out(), "Autostart is currently enabled.\n");
}
