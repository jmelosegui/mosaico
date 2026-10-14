use super::*;
use crate::commands::context::Exit;
use crate::commands::fake_desktop::monitor;
use crate::commands::test_support::{Harness, Lifetime};

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
