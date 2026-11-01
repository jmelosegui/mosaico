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
    let config_dir = h.config.dir().display().to_string();
    assert!(out.contains(&config_dir), "{out}");
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
fn tells_about_a_newer_release_once_started() {
    // Arrange
    let mut h = Harness::new("start-newer-release");
    h.world.releases.publish("v99.0.0");

    // Act
    let exit = h.run(execute);

    // Assert
    assert_eq!(exit, Exit::Success);
    let out = h.out();
    let local = env!("CARGO_PKG_VERSION");
    assert!(
        out.contains(&format!("Update available: v{local} -> v99.0.0")),
        "{out}"
    );
    assert!(out.contains("Run 'mosaico update' to install it"), "{out}");
}

#[test]
fn says_nothing_about_updates_on_the_latest_release() {
    // Arrange
    let mut h = Harness::new("start-latest-release");
    h.world
        .releases
        .publish(&format!("v{}", env!("CARGO_PKG_VERSION")));

    // Act
    let exit = h.run(execute);

    // Assert
    assert_eq!(exit, Exit::Success);
    assert!(!h.out().contains("Update available"), "{}", h.out());
}

#[test]
fn says_nothing_about_updates_when_github_cannot_be_reached() {
    // Arrange -- nothing is published, so the lookup fails.
    let mut h = Harness::new("start-offline");

    // Act
    let exit = h.run(execute);

    // Assert
    assert_eq!(exit, Exit::Success);
    assert!(!h.out().contains("Update available"), "{}", h.out());
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

#[test]
fn quietly_starts_the_daemon_without_a_banner_or_looking_for_a_release() {
    // Arrange
    let mut h = Harness::new("start-quiet");
    *h.world.launch.borrow_mut() = Launch::Answers(4321);
    h.world.releases.publish("v99.0.0");

    // Act
    let exit = h.run(quietly);

    // Assert
    assert_eq!(exit, Exit::Success);
    assert!(h.world.daemon_running.get());
    assert_eq!(h.out(), "");
    assert_eq!(h.world.releases.requests.get(), 0, "no update check");
}

#[test]
fn quietly_says_nothing_when_the_daemon_is_already_running() {
    // Arrange
    let mut h = Harness::new("start-quiet-running");
    h.world.daemon_up(10);

    // Act
    let exit = h.run(quietly);

    // Assert
    assert_eq!(exit, Exit::Success);
    assert_eq!(h.out(), "");
    assert_eq!(h.world.processes.borrow().len(), 1, "nothing launched");
}

#[test]
fn quietly_still_reports_a_daemon_that_exits_right_after_starting() {
    // Arrange
    let mut h = Harness::new("start-quiet-rejected");
    *h.world.launch.borrow_mut() = Launch::ExitsAtOnce(4321);

    // Act
    let exit = h.run(quietly);

    // Assert
    assert_eq!(exit, Exit::Failure);
    assert!(h.err().contains("exited immediately"), "{}", h.err());
}
