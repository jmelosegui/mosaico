use std::time::Duration;

use mosaico_core::ipc::{DaemonStatus, ErrorCode, IpcError, Response};

use super::*;
use crate::commands::test_support::{Harness, Lifetime, World};

#[test]
fn reports_when_nothing_is_running() {
    // Arrange
    let mut h = Harness::new("stop-nothing");

    // Act
    let exit = h.run(execute);

    // Assert
    assert_eq!(exit, Exit::Success);
    assert_eq!(h.out(), "Mosaico is not running.\n");
    assert!(h.world.sent.borrow().is_empty());
}

#[test]
fn stops_the_daemon_and_waits_for_its_process_to_exit() {
    // Arrange -- the process takes a few checks to unwind after the stop.
    let mut h = Harness::new("stop-graceful");
    h.world.daemon_up(77);
    h.world
        .processes
        .borrow_mut()
        .insert(77, Lifetime::Checks(3));
    h.pid_file.write(77).unwrap();
    h.world
        .reply(Ok(Response::ok_with_message("Daemon stopping")));

    // Act
    let exit = h.run(execute);

    // Assert
    assert_eq!(exit, Exit::Success);
    assert_eq!(*h.world.sent.borrow(), vec![Command::Stop]);
    assert_eq!(h.out(), "Mosaico stopped. Daemon stopping\n");
    assert_eq!(h.err(), "", "no warning when it exits in time");
    assert_eq!(h.pid_file.read().unwrap(), None);
}

#[test]
fn warns_about_a_daemon_that_never_exits() {
    // Arrange
    let mut h = Harness::new("stop-wedged");
    h.world.daemon_up(77);
    h.pid_file.write(77).unwrap();
    let start = h.world.now.get();

    // Act
    let exit = h.run(execute);

    // Assert -- gave up after the timeout instead of blocking.
    assert_eq!(exit, Exit::Success);
    assert!(h.err().contains("still running after 10s"), "{}", h.err());
    assert!(h.world.now.get() - start >= SHUTDOWN_TIMEOUT);
}

#[test]
fn kills_a_daemon_whose_channel_is_gone() {
    // Arrange -- the IPC thread died but the process lives on.
    let mut h = Harness::new("stop-kill");
    h.world.processes.borrow_mut().insert(88, Lifetime::Forever);
    h.pid_file.write(88).unwrap();

    // Act
    let exit = h.run(execute);

    // Assert
    assert_eq!(exit, Exit::Success);
    assert_eq!(h.out(), "Mosaico stopped (killed PID 88).\n");
    assert!(!h.world.processes.borrow().contains_key(&88));
    assert_eq!(h.pid_file.read().unwrap(), None);
}

#[test]
fn fails_when_the_process_cannot_be_killed() {
    // Arrange
    let mut h = Harness::new("stop-unkillable");
    h.world.processes.borrow_mut().insert(88, Lifetime::Forever);
    h.world.can_kill.set(false);
    h.pid_file.write(88).unwrap();

    // Act
    let exit = h.run(execute);

    // Assert
    assert_eq!(exit, Exit::Failure);
    assert_eq!(h.err(), "Failed to kill process 88.\n");
}

#[test]
fn a_process_that_could_not_be_killed_keeps_its_pid_file() {
    // Arrange -- ending it failed, so it is still running.
    let mut h = Harness::new("stop-unkillable-pid");
    h.world.processes.borrow_mut().insert(88, Lifetime::Forever);
    h.world.can_kill.set(false);
    h.pid_file.write(88).unwrap();

    // Act
    let exit = h.run(execute);

    // Assert -- status and a later stop can still find it.
    assert_eq!(exit, Exit::Failure);
    assert_eq!(h.pid_file.read().unwrap(), Some(88));
    assert_eq!(h.out(), "", "it must not be reported as stopped");
}

#[test]
fn a_process_that_exits_just_before_the_kill_is_a_failed_kill() {
    // Arrange -- alive at the liveness check, gone by the time it is
    // ended, which makes ending it fail.
    let mut h = Harness::new("stop-exits-first");
    h.world
        .processes
        .borrow_mut()
        .insert(88, Lifetime::Checks(1));
    h.pid_file.write(88).unwrap();

    // Act
    let exit = h.run(execute);

    // Assert -- the same as on a failed kill.
    assert_eq!(exit, Exit::Failure);
    assert_eq!(h.err(), "Failed to kill process 88.\n");
    assert_eq!(h.pid_file.read().unwrap(), Some(88));
}

#[test]
fn a_stale_pid_file_is_not_mistaken_for_a_daemon() {
    // Arrange -- a daemon that crashed left its pid file behind.
    let mut h = Harness::new("stop-stale");
    h.pid_file.write(99).unwrap();

    // Act
    let exit = h.run(execute);

    // Assert
    assert_eq!(exit, Exit::Success);
    assert_eq!(h.out(), "Mosaico is not running.\n");
}

#[test]
fn reports_a_refused_stop_with_what_to_do() {
    // Arrange
    let mut h = Harness::new("stop-refused");
    h.world.daemon_up(77);
    let status = DaemonStatus {
        version: "0.0.1".into(),
        protocol: 99,
        windows: 0,
    };
    h.world.reply(Ok(
        Response::error(ErrorCode::VersionMismatch, "mismatch").with_data(&status)
    ));

    // Act
    let exit = h.run(execute);

    // Assert
    assert_eq!(exit, Exit::Failure);
    assert!(h.err().contains("version 0.0.1"), "{}", h.err());
}

#[test]
fn falls_back_to_the_pid_file_when_the_stop_request_fails() {
    // Arrange -- the daemon answers the liveness check, then the
    // request itself breaks.
    let mut h = Harness::new("stop-ipc-error");
    h.world.daemon_up(77);
    h.pid_file.write(77).unwrap();
    h.world.reply(Err(IpcError::Closed));

    // Act
    let exit = h.run(execute);

    // Assert
    assert_eq!(exit, Exit::Success);
    assert!(h.err().contains("IPC failed"), "{}", h.err());
    assert_eq!(h.out(), "Mosaico stopped (killed PID 77).\n");
}

// -- wait_until_gone --

#[test]
fn returns_immediately_when_the_process_is_already_gone() {
    // Arrange
    let world = World::default();
    let start = world.now.get();
    let mut calls = 0;

    // Act
    let gone = wait_until_gone(
        || {
            calls += 1;
            false
        },
        &world,
        Duration::from_secs(10),
        Duration::from_millis(1),
    );

    // Assert -- one check, no sleeping.
    assert!(gone);
    assert_eq!(calls, 1);
    assert_eq!(world.now.get(), start);
}

#[test]
fn waits_for_a_process_that_is_still_unwinding() {
    // Arrange
    let world = World::default();
    let mut calls = 0;

    // Act
    let gone = wait_until_gone(
        || {
            calls += 1;
            calls < 4
        },
        &world,
        Duration::from_secs(10),
        Duration::from_millis(1),
    );

    // Assert
    assert!(gone);
    assert_eq!(calls, 4);
}

#[test]
fn gives_up_on_a_daemon_that_never_exits() {
    // Arrange
    let world = World::default();
    let start = world.now.get();

    // Act
    let gone = wait_until_gone(
        || true,
        &world,
        Duration::from_millis(60),
        Duration::from_millis(10),
    );

    // Assert -- reports failure rather than blocking, and not before
    // the timeout either.
    assert!(!gone);
    assert!(world.now.get() - start >= Duration::from_millis(60));
}

#[test]
fn a_daemon_that_does_not_reply_is_killed_from_its_pid_file() {
    // #86
    // Arrange -- the daemon is listening but never answers the stop.
    let mut h = Harness::new("stop-unresponsive");
    h.world.daemon_up(77);
    h.pid_file.write(77).unwrap();
    h.world.reply(Err(IpcError::TimedOut));

    // Act
    let exit = h.run(execute);

    // Assert
    assert_eq!(exit, Exit::Success);
    assert_eq!(h.out(), "Mosaico stopped (killed PID 77).\n");
    assert!(!h.world.processes.borrow().contains_key(&77));
}
