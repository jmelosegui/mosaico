use mosaico_core::ipc::{Command, ErrorCode, Response};

use super::*;
use crate::commands::test_support::Harness;

fn reloaded_then_paused() -> Vec<DesktopEvent> {
    vec![
        DesktopEvent::ConfigReloaded,
        DesktopEvent::Paused { paused: true },
    ]
}

#[test]
fn prints_each_event_until_the_daemon_stops() {
    // Arrange
    let mut h = Harness::new("events-text");
    h.world.daemon_up(1);
    *h.world.events.borrow_mut() = reloaded_then_paused();

    // Act
    let exit = h.run(|cx| execute(cx, false, &[]));

    // Assert
    assert_eq!(exit, Exit::Success);
    assert_eq!(h.out(), "config_reloaded\npaused paused=true\n");
}

#[test]
fn prints_json_lines_when_asked() {
    // Arrange
    let mut h = Harness::new("events-json");
    h.world.daemon_up(1);
    *h.world.events.borrow_mut() = reloaded_then_paused();

    // Act
    let exit = h.run(|cx| execute(cx, true, &[]));

    // Assert
    assert_eq!(exit, Exit::Success);
    assert_eq!(
        h.out(),
        "{\"event\":\"config_reloaded\"}\n{\"event\":\"paused\",\"paused\":true}\n"
    );
}

#[test]
fn subscribes_to_the_kinds_named() {
    // Arrange
    let mut h = Harness::new("events-kinds");
    h.world.daemon_up(1);

    // Act
    let exit = h.run(|cx| execute(cx, false, &["focus_changed".to_string()]));

    // Assert
    assert_eq!(exit, Exit::Success);
    assert_eq!(
        *h.world.sent.borrow(),
        vec![Command::Subscribe {
            events: vec![EventKind::FocusChanged]
        }]
    );
}

#[test]
fn refuses_an_unknown_kind_without_asking_the_daemon() {
    // Arrange
    let mut h = Harness::new("events-unknown");
    h.world.daemon_up(1);

    // Act
    let exit = h.run(|cx| execute(cx, false, &["focus".to_string()]));

    // Assert
    assert_eq!(exit, Exit::Failure);
    assert!(
        h.err().starts_with("Error: unknown event `focus`"),
        "{}",
        h.err()
    );
    assert!(h.world.sent.borrow().is_empty());
}

#[test]
fn says_so_when_it_fell_behind() {
    // Arrange
    let mut h = Harness::new("events-lagged");
    h.world.daemon_up(1);
    *h.world.events.borrow_mut() = vec![DesktopEvent::Lagged];

    // Act
    let exit = h.run(|cx| execute(cx, false, &[]));

    // Assert
    assert_eq!(exit, Exit::Failure);
    assert_eq!(h.out(), "lagged\n");
    assert!(h.err().contains("Fell behind"), "{}", h.err());
}

#[test]
fn reports_a_refusal_and_a_missing_daemon() {
    // Arrange
    let mut refused = Harness::new("events-refused");
    refused.world.daemon_up(1);
    refused.world.reply(Ok(Response::error(
        ErrorCode::UnknownCommand,
        "unknown command",
    )));
    let mut down = Harness::new("events-down");

    // Act
    let refused_exit = refused.run(|cx| execute(cx, false, &[]));
    let down_exit = down.run(|cx| execute(cx, false, &[]));

    // Assert
    assert_eq!(refused_exit, Exit::Failure);
    assert_eq!(refused.err(), "Error: unknown command\n");
    assert_eq!(down_exit, Exit::Failure);
    assert_eq!(down.err(), "Mosaico is not running.\n");
}
