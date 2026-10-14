use mosaico_core::Action;
use mosaico_core::action::Direction;

use super::*;
use crate::commands::test_support::Harness;

#[test]
fn prints_the_reply_of_a_successful_command() {
    // Arrange
    let mut h = Harness::new("request-ok");
    h.world.daemon_up(1);
    h.world.reply(Ok(Response::ok_with_message("state dump")));

    // Act
    let exit = h.run(|cx| run(cx, &Command::Inspect));

    // Assert
    assert_eq!(exit, Exit::Success);
    assert_eq!(h.out(), "state dump\n");
}

#[test]
fn fails_when_no_daemon_is_running() {
    // Arrange
    let mut h = Harness::new("request-down");

    // Act
    let exit = h.run(|cx| run(cx, &Command::Inspect));

    // Assert
    assert_eq!(exit, Exit::Failure);
    assert_eq!(h.err(), "Mosaico is not running.\n");
}

#[test]
fn reports_a_refusal_with_the_daemon_message() {
    // Arrange
    let mut h = Harness::new("request-refused");
    h.world.daemon_up(1);
    h.world
        .reply(Ok(Response::error(ErrorCode::Failed, "nothing to focus")));

    // Act
    let exit = h.run(|cx| run(cx, &Command::Inspect));

    // Assert
    assert_eq!(exit, Exit::Failure);
    assert_eq!(h.err(), "Error: nothing to focus\n");
}

#[test]
fn tells_the_user_to_restart_a_mismatched_daemon() {
    // Arrange
    let mut h = Harness::new("request-mismatch");
    h.world.daemon_up(1);
    let status = DaemonStatus {
        version: "0.0.1".into(),
        protocol: 99,
        windows: 0,
    };
    h.world.reply(Ok(
        Response::error(ErrorCode::VersionMismatch, "mismatch").with_data(&status)
    ));

    // Act
    let exit = h.run(|cx| run(cx, &Command::Status));

    // Assert
    assert_eq!(exit, Exit::Failure);
    let err = h.err();
    assert!(err.contains("version 0.0.1"), "{err}");
    assert!(err.contains("mosaico stop"), "{err}");
}

#[test]
fn reports_a_broken_channel() {
    // Arrange -- the transport itself failed.
    let mut h = Harness::new("request-broken");
    h.world.daemon_up(1);
    h.world
        .reply(Err(IpcError::Io(std::io::Error::other("the pipe broke"))));

    // Act
    let exit = h.run(|cx| run(cx, &Command::Status));

    // Assert
    assert_eq!(exit, Exit::Failure);
    assert_eq!(h.err(), "Failed to reach the daemon: the pipe broke\n");
}

#[test]
fn suggests_a_restart_when_the_daemon_closes_without_replying() {
    // Arrange -- a daemon older than the protocol drops a request it
    // cannot read.
    let mut h = Harness::new("request-closed");
    h.world.daemon_up(1);
    h.world.reply(Err(IpcError::Closed));

    // Act
    let exit = h.run(|cx| run(cx, &Command::Inspect));

    // Assert
    assert_eq!(exit, Exit::Failure);
    assert_eq!(h.err(), format!("{CLOSED_WITHOUT_REPLY}\n"));
}

#[test]
fn each_command_sends_what_it_says() {
    // Arrange
    let mut h = Harness::new("request-commands");
    h.world.daemon_up(1);
    let action = Action::Focus(Direction::Left);

    // Act
    h.run(|cx| crate::commands::action::execute(cx, action));
    h.run(crate::commands::pause::pause);
    h.run(crate::commands::pause::unpause);
    h.run(crate::commands::debug::state::execute);

    // Assert
    assert_eq!(
        *h.world.sent.borrow(),
        vec![
            Command::Action { action },
            Command::PauseHotkeys,
            Command::UnpauseHotkeys,
            Command::Inspect,
        ]
    );
}

#[test]
fn a_daemon_busy_with_another_client_still_gets_the_command() {
    // Arrange -- its liveness check misses it, but it answers (#73).
    let mut h = Harness::new("request-busy");
    h.world.daemon_up(1);
    h.world.looks_missing.set(true);
    h.world.reply(Ok(Response::ok_with_message("state dump")));

    // Act
    let exit = h.run(|cx| run(cx, &Command::Inspect));

    // Assert
    assert_eq!(exit, Exit::Success);
    assert_eq!(h.out(), "state dump\n");
}

#[test]
fn status_and_stop_reach_a_busy_daemon() {
    // Arrange
    let mut h = Harness::new("request-busy-status-stop");
    h.world.daemon_up(1);
    h.world.looks_missing.set(true);

    // Act
    h.run(crate::commands::status::execute);
    let stopped = h.run(crate::commands::stop::execute);

    // Assert
    assert!(h.out().starts_with("Mosaico is running"), "{}", h.out());
    assert!(h.out().contains("Mosaico stopped."), "{}", h.out());
    assert_eq!(stopped, Exit::Success);
    assert_eq!(*h.world.sent.borrow(), vec![Command::Status, Command::Stop]);
}

#[test]
fn an_unresponsive_daemon_is_reported_with_what_to_do() {
    // Arrange -- #86: the daemon is listening but does not reply.
    let mut h = Harness::new("request-unresponsive");
    h.world.daemon_up(1);
    h.world.reply(Err(IpcError::TimedOut));

    // Act
    let exit = h.run(|cx| run(cx, &Command::Inspect));

    // Assert
    assert_eq!(exit, Exit::Failure);
    assert_eq!(h.err(), format!("{NOT_RESPONDING}\n"));
}

#[test]
fn stop_says_it_is_ending_an_unresponsive_daemon() {
    // Arrange
    let mut h = Harness::new("request-unresponsive-stop");
    h.world.daemon_up(77);
    h.pid_file.write(77).unwrap();
    h.world.reply(Err(IpcError::TimedOut));

    // Act
    h.run(crate::commands::stop::execute);

    // Assert
    assert_eq!(h.err(), "The daemon is not responding; ending it.\n");
}
