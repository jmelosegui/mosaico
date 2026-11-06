use std::path::PathBuf;

use mosaico_core::ipc::Command;
use mosaico_core::system::LaunchCommand;

use super::*;
use crate::commands::test_support::{EXE, Harness, SETTINGS};

/// The settings app beside the executable the hooks run from.
fn settings() -> PathBuf {
    PathBuf::from(EXE).with_file_name(SETTINGS)
}

#[test]
fn before_an_upgrade_a_running_daemon_is_stopped_and_noted_for_restart() {
    // Arrange
    let mut h = Harness::new("hook-upgrade-running");
    h.world.daemon_up(4321);

    // Act
    let exit = h.run(|cx| execute(cx, Hook::BeforeUpgrade));

    // Assert
    assert_eq!(exit, Exit::Success);
    assert_eq!(*h.world.sent.borrow(), [Command::Stop]);
    assert!(!h.world.daemon_running.get());
    assert!(h.restart.take().unwrap());
}

#[test]
fn before_an_upgrade_a_stopped_daemon_is_not_noted_for_restart() {
    // Arrange
    let mut h = Harness::new("hook-upgrade-stopped");

    // Act
    let exit = h.run(|cx| execute(cx, Hook::BeforeUpgrade));

    // Assert
    assert_eq!(exit, Exit::Success);
    assert!(!h.restart.take().unwrap());
}

#[test]
fn before_an_upgrade_a_running_settings_app_is_moved_aside() {
    // Arrange
    let mut h = Harness::new("hook-upgrade-settings");
    h.world.installer.running.borrow_mut().push(settings());

    // Act
    let exit = h.run(|cx| execute(cx, Hook::BeforeUpgrade));

    // Assert
    assert_eq!(exit, Exit::Success);
    assert_eq!(*h.world.installer.set_aside.borrow(), [settings()]);
    assert!(h.out().contains("Moved the running settings app aside.\n"));
}

#[test]
fn after_an_install_the_daemon_stopped_for_it_starts_again() {
    // Arrange
    let mut h = Harness::new("hook-install-restart");
    h.restart.set().unwrap();

    // Act
    let exit = h.run(|cx| execute(cx, Hook::AfterInstall));

    // Assert
    assert_eq!(exit, Exit::Success);
    assert!(h.world.daemon_running.get());
    assert!(!h.restart.take().unwrap());
}

#[test]
fn after_an_install_a_daemon_that_was_not_running_stays_stopped() {
    // Arrange
    let mut h = Harness::new("hook-install-fresh");

    // Act
    let exit = h.run(|cx| execute(cx, Hook::AfterInstall));

    // Assert
    assert_eq!(exit, Exit::Success);
    assert!(!h.world.daemon_running.get());
    assert_eq!(h.out(), "");
}

#[test]
fn before_an_uninstall_the_daemon_stops_and_its_autostart_is_removed() {
    // Arrange
    let mut h = Harness::new("hook-uninstall");
    h.world.daemon_up(4321);
    *h.world.autostart.borrow_mut() = Some(LaunchCommand::autostart(EXE));
    h.world.installer.running.borrow_mut().push(settings());
    h.restart.set().unwrap();

    // Act
    let exit = h.run(|cx| execute(cx, Hook::BeforeUninstall));

    // Assert
    assert_eq!(exit, Exit::Success, "{}", h.err());
    assert!(!h.world.daemon_running.get());
    assert_eq!(*h.world.autostart.borrow(), None);
    assert_eq!(*h.world.installer.set_aside.borrow(), [settings()]);
    assert_eq!(*h.world.installer.cleaned.borrow(), [PathBuf::from(EXE)]);
    assert!(!h.restart.take().unwrap());
}

#[test]
fn before_an_uninstall_the_autostart_of_another_mosaico_stays() {
    // Arrange
    let mut h = Harness::new("hook-uninstall-other");
    let other = LaunchCommand::autostart(r"C:\other\mosaico.exe");
    *h.world.autostart.borrow_mut() = Some(other.clone());
    h.world.files.borrow_mut().push(other.program.clone());

    // Act
    let exit = h.run(|cx| execute(cx, Hook::BeforeUninstall));

    // Assert
    assert_eq!(exit, Exit::Success, "{}", h.err());
    assert_eq!(*h.world.autostart.borrow(), Some(other));
}

#[test]
fn before_an_uninstall_the_turned_off_autostart_of_another_mosaico_stays() {
    // Arrange
    let mut h = Harness::new("hook-uninstall-other-off");
    let other = LaunchCommand::autostart(r"C:\other\mosaico.exe");
    *h.world.autostart.borrow_mut() = Some(other.clone());
    h.world.files.borrow_mut().push(other.program.clone());
    h.world.turned_off.set(true);

    // Act
    let exit = h.run(|cx| execute(cx, Hook::BeforeUninstall));

    // Assert
    assert_eq!(exit, Exit::Success, "{}", h.err());
    assert_eq!(*h.world.autostart.borrow(), Some(other));
}

#[test]
fn before_an_uninstall_an_autostart_of_this_mosaico_with_other_arguments_is_removed() {
    // Arrange
    let mut h = Harness::new("hook-uninstall-args");
    let mut this = LaunchCommand::autostart(EXE);
    this.args.push("--verbose".into());
    *h.world.autostart.borrow_mut() = Some(this);

    // Act
    let exit = h.run(|cx| execute(cx, Hook::BeforeUninstall));

    // Assert
    assert_eq!(exit, Exit::Success, "{}", h.err());
    assert_eq!(*h.world.autostart.borrow(), None);
}
