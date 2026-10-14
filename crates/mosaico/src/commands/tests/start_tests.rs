use super::*;
use crate::commands::test_support::{Harness, Launch, Lifetime};

#[test]
fn does_nothing_when_the_daemon_is_already_running() {
    // Arrange
    let mut h = Harness::new("start-running");
    h.world.daemon_up(10);

    // Act
    let exit = h.run(execute);

    // Assert
    assert_eq!(exit, Exit::Success);
    assert_eq!(h.out(), "Mosaico is already running.\n");
    assert_eq!(h.world.processes.borrow().len(), 1, "nothing launched");
}

#[test]
fn starts_the_daemon_and_shows_where_its_config_lives() {
    // Arrange
    let mut h = Harness::new("start-ok");
    *h.world.launch.borrow_mut() = Launch::Answers(4321);

    // Act
    let exit = h.run(execute);

    // Assert
    assert_eq!(exit, Exit::Success);
    let out = h.out();
    assert!(out.contains("Started (PID: \x1b[1;97m4321"), "{out}");
    assert!(out.contains("C:\\config\\mosaico"), "{out}");
}

#[test]
fn clears_a_stale_pid_file_before_starting() {
    // Arrange -- a daemon that crashed left its pid file behind.
    let mut h = Harness::new("start-stale");
    h.pid_file.write(99).unwrap();

    // Act
    let exit = h.run(execute);

    // Assert
    assert_eq!(exit, Exit::Success);
    assert!(h.world.daemon_running.get());
    assert_eq!(h.pid_file.read().unwrap(), None);
}

#[test]
fn does_not_start_a_second_daemon_beside_an_unresponsive_one() {
    // Arrange -- the process is alive but stopped answering.
    let mut h = Harness::new("start-unresponsive");
    h.world.processes.borrow_mut().insert(99, Lifetime::Forever);
    h.pid_file.write(99).unwrap();

    // Act
    let exit = h.run(execute);

    // Assert
    assert_eq!(exit, Exit::Success);
    assert_eq!(
        h.out(),
        "Mosaico process exists (PID: 99) but is not responding.\n"
    );
    assert!(!h.world.daemon_running.get(), "nothing launched");
}

#[test]
fn reports_a_daemon_that_exits_right_after_starting() {
    // Arrange -- rejected by the lock of a daemon still shutting down.
    let mut h = Harness::new("start-rejected");
    *h.world.launch.borrow_mut() = Launch::ExitsAtOnce(4321);

    // Act
    let exit = h.run(execute);

    // Assert
    assert_eq!(exit, Exit::Failure);
    assert!(h.err().contains("exited immediately"), "{}", h.err());
    assert_eq!(h.out(), "", "no banner for a daemon that is not there");
}

#[test]
fn gives_up_on_a_daemon_that_never_answers() {
    // Arrange
    let mut h = Harness::new("start-silent");
    *h.world.launch.borrow_mut() = Launch::Silent(4321);
    let start = h.world.now.get();

    // Act
    let exit = h.run(execute);

    // Assert
    assert_eq!(exit, Exit::Failure);
    assert!(h.world.now.get() - start >= STARTUP_TIMEOUT);
}

#[test]
fn reports_a_daemon_that_cannot_be_launched() {
    // Arrange
    let mut h = Harness::new("start-launch-fails");
    *h.world.launch.borrow_mut() = Launch::Fails("no such file".into());

    // Act
    let exit = h.run(execute);

    // Assert
    assert_eq!(exit, Exit::Failure);
    assert_eq!(h.err(), "Error: no such file\n");
}
