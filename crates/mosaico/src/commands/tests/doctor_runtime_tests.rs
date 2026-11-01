use std::path::PathBuf;

use mosaico_core::system::LaunchCommand;

use super::*;
use crate::commands::context::Exit;
use crate::commands::fake_desktop::monitor;
use crate::commands::test_support::{EXE, Harness, Lifetime, MOVED_AWAY};

#[test]
fn a_running_daemon_is_reported_with_its_pid() {
    // Arrange
    let mut h = Harness::new("doctor-running");
    h.world.daemon_up(42);
    h.pid_file.write(42).unwrap();

    // Act
    h.run(|cx| {
        check_daemon(cx);
        Exit::Success
    });

    // Assert
    assert!(
        h.out().contains("Daemon is running (PID: 42)"),
        "{}",
        h.out()
    );
}

#[test]
fn a_stale_pid_file_is_removed() {
    // Arrange
    let mut h = Harness::new("doctor-stale");
    h.pid_file.write(42).unwrap();

    // Act
    h.run(|cx| {
        check_daemon(cx);
        Exit::Success
    });

    // Assert
    assert!(
        h.out().contains("Removed stale PID file (PID: 42)"),
        "{}",
        h.out()
    );
    assert_eq!(h.pid_file.read().unwrap(), None);
}

#[test]
fn a_hung_daemon_is_flagged() {
    // Arrange
    let mut h = Harness::new("doctor-hung");
    h.world.processes.borrow_mut().insert(42, Lifetime::Forever);
    h.pid_file.write(42).unwrap();

    // Act
    h.run(|cx| {
        check_daemon(cx);
        Exit::Success
    });

    // Assert
    assert!(h.out().contains("not responding"), "{}", h.out());
}

#[test]
fn monitors_are_reported_left_to_right_with_their_work_areas() {
    // Arrange -- the platform finds the right monitor first.
    let mut h = Harness::new("doctor-monitors");
    h.desktop.monitors = vec![monitor(2, 1920), monitor(1, 0)];

    // Act
    h.run(|cx| {
        check_monitors(cx);
        Exit::Success
    });

    // Assert
    assert_eq!(
        h.out(),
        format!(
            "  {OK} 2 monitor(s) detected\n       \
             Monitor 0: 1920x1040 at (0, 0)\n       \
             Monitor 1: 1920x1040 at (1920, 0)\n"
        )
    );
}

#[test]
fn no_monitors_is_a_failure() {
    // Arrange
    let mut h = Harness::new("doctor-no-monitors");

    // Act
    h.run(|cx| {
        check_monitors(cx);
        Exit::Success
    });

    // Assert
    assert_eq!(h.out(), format!("  {FAIL} No monitors detected\n"));
}

#[test]
fn monitors_that_cannot_be_listed_are_reported_with_the_reason() {
    // Arrange
    let mut h = Harness::new("doctor-monitors-unreadable");
    h.desktop.monitors_unreadable = true;

    // Act
    h.run(|cx| {
        check_monitors(cx);
        Exit::Success
    });

    // Assert
    assert_eq!(
        h.out(),
        format!("  {FAIL} Could not enumerate monitors: could not list monitors: access denied\n")
    );
}

/// What doctor's autostart check prints.
fn autostart_check(h: &mut Harness) -> String {
    h.run(|cx| {
        check_autostart(cx);
        Exit::Success
    });
    h.out()
}

#[test]
fn an_autostart_entry_that_cannot_be_read_fails() {
    // Arrange
    let mut h = Harness::new("doctor-autostart-unreadable");
    *h.world.autostart_error.borrow_mut() = Some("access denied".into());

    // Act
    let out = autostart_check(&mut h);

    // Assert
    assert_eq!(out, format!("  {FAIL} Autostart: access denied\n"));
}

#[test]
fn an_autostart_entry_for_this_mosaico_is_ok() {
    // Arrange
    let mut h = Harness::new("doctor-autostart-ok");
    *h.world.autostart.borrow_mut() = Some(LaunchCommand::autostart(EXE));

    // Act
    let out = autostart_check(&mut h);

    // Assert
    assert_eq!(out, format!("  {OK} Autostart is enabled\n"));
}

#[test]
fn no_autostart_entry_is_a_warning() {
    // Arrange
    let mut h = Harness::new("doctor-autostart-none");

    // Act
    let out = autostart_check(&mut h);

    // Assert
    assert_eq!(
        out,
        format!("  {WARN} Autostart is disabled (run `mosaico autostart enable` to set up)\n")
    );
}

#[test]
fn an_autostart_entry_with_other_arguments_is_a_warning() {
    // Arrange
    let mut h = Harness::new("doctor-autostart-other");
    *h.world.autostart.borrow_mut() = Some(LaunchCommand {
        program: EXE.into(),
        args: vec!["daemon".into()],
    });

    // Act
    let out = autostart_check(&mut h);

    // Assert
    assert_eq!(
        out,
        format!(
            "  {WARN} Autostart starts \"{EXE}\" daemon instead of \"{EXE}\" start --quiet (run `mosaico autostart enable` to switch)\n"
        )
    );
}

#[test]
fn an_autostart_entry_from_before_quiet_starts_is_a_warning() {
    // Arrange -- what older releases registered.
    let mut h = Harness::new("doctor-autostart-old");
    *h.world.autostart.borrow_mut() = Some(LaunchCommand {
        program: EXE.into(),
        args: vec!["start".into()],
    });

    // Act
    let out = autostart_check(&mut h);

    // Assert
    assert_eq!(
        out,
        format!(
            "  {WARN} Autostart starts \"{EXE}\" start instead of \"{EXE}\" start --quiet (run `mosaico autostart enable` to switch)\n"
        )
    );
}

#[test]
fn an_autostart_entry_for_another_mosaico_is_a_warning() {
    // Arrange
    let mut h = Harness::new("doctor-autostart-another");
    let other = r"C:\dev\mosaico\mosaico.exe";
    h.world.files.borrow_mut().push(other.into());
    *h.world.autostart.borrow_mut() = Some(LaunchCommand::autostart(other));

    // Act
    let out = autostart_check(&mut h);

    // Assert
    assert_eq!(
        out,
        format!(
            "  {WARN} Autostart starts \"{other}\" start --quiet instead of \"{EXE}\" start --quiet (run `mosaico autostart enable` to switch)\n"
        )
    );
}

#[test]
fn an_autostart_entry_for_a_program_that_is_not_a_file_fails() {
    // Arrange
    let mut h = Harness::new("doctor-autostart-missing");
    *h.world.autostart.borrow_mut() = Some(LaunchCommand::autostart(MOVED_AWAY));

    // Act
    let out = autostart_check(&mut h);

    // Assert
    assert_eq!(
        out,
        format!(
            "  {FAIL} Autostart starts \"{MOVED_AWAY}\", which is not a file (run `mosaico autostart enable` to fix)\n"
        )
    );
}

#[test]
fn an_empty_autostart_entry_fails() {
    // Arrange
    let mut h = Harness::new("doctor-autostart-empty");
    *h.world.autostart.borrow_mut() = Some(LaunchCommand {
        program: PathBuf::new(),
        args: Vec::new(),
    });

    // Act
    let out = autostart_check(&mut h);

    // Assert
    assert_eq!(
        out,
        format!(
            "  {FAIL} The autostart entry is empty (run `mosaico autostart enable` to set it, or `mosaico autostart disable` to remove it)\n"
        )
    );
}

#[test]
fn an_autostart_entry_the_user_turned_off_is_a_warning() {
    // Arrange
    let mut h = Harness::new("doctor-autostart-turned-off");
    *h.world.autostart.borrow_mut() = Some(LaunchCommand::autostart(EXE));
    h.world.turned_off.set(true);

    // Act
    let out = autostart_check(&mut h);

    // Assert
    assert_eq!(
        out,
        format!(
            "  {WARN} Autostart is turned off (run `mosaico autostart enable` to turn it back on)\n"
        )
    );
}
