use mosaico_core::system::LaunchCommand;

use super::*;
use crate::commands::test_support::{EXE, Harness, MOVED_AWAY};

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

#[test]
fn enabling_replaces_an_entry_for_another_mosaico() {
    // Arrange
    let mut h = Harness::new("autostart-enable-replaces");
    *h.world.autostart.borrow_mut() = Some(LaunchCommand::autostart(r"C:\old\mosaico.exe"));

    // Act
    let exit = h.run(enable);

    // Assert
    assert_eq!(exit, Exit::Success);
    assert_eq!(
        *h.world.autostart.borrow(),
        Some(LaunchCommand::autostart(EXE))
    );
}

#[test]
fn status_names_the_command_of_another_registration() {
    // Arrange
    let mut h = Harness::new("autostart-status-other");
    *h.world.autostart.borrow_mut() = Some(LaunchCommand {
        program: EXE.into(),
        args: vec!["daemon".into()],
    });

    // Act
    let exit = h.run(status);

    // Assert
    assert_eq!(exit, Exit::Success);
    assert_eq!(
        h.out(),
        format!(
            "Autostart is currently enabled, but starts \"{EXE}\" daemon instead of \"{EXE}\" start.\n"
        )
    );
}

#[test]
fn status_says_when_the_registered_program_is_not_a_file() {
    // Arrange
    let mut h = Harness::new("autostart-status-missing");
    *h.world.autostart.borrow_mut() = Some(LaunchCommand::autostart(MOVED_AWAY));

    // Act
    let exit = h.run(status);

    // Assert
    assert_eq!(exit, Exit::Success);
    assert_eq!(
        h.out(),
        format!("Autostart is currently enabled, but \"{MOVED_AWAY}\" is not a file.\n")
    );
}

#[test]
fn status_with_nothing_registered_does_not_need_the_executable() {
    // Arrange
    let mut h = Harness::new("autostart-status-no-exe");
    h.world.exe_missing.set(true);

    // Act
    let exit = h.run(status);

    // Assert
    assert_eq!(exit, Exit::Success);
    assert_eq!(h.out(), "Autostart is currently disabled.\n");
}

#[test]
fn status_fails_when_the_executable_cannot_be_found() {
    // Arrange
    let mut h = Harness::new("autostart-status-lost-exe");
    h.world.exe_missing.set(true);
    *h.world.autostart.borrow_mut() = Some(LaunchCommand::autostart(EXE));

    // Act
    let exit = h.run(status);

    // Assert
    assert_eq!(exit, Exit::Failure);
    assert_eq!(
        h.err(),
        "Error: could not find the mosaico executable: no executable\n"
    );
}
