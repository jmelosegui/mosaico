use std::time::Duration;

use serde_json::json;

use crate::api::{Limits, LoopRequest};
use crate::ipc::framing::MAX_MESSAGE_BYTES;

use super::session_world::{HELLO, World, state};

#[test]
fn a_bar_speaking_no_version_in_common_is_told_which_the_daemon_speaks() {
    // Arrange
    let world = World::new();
    let mut bar = world.listener.connect();

    // Act
    bar.send(r#"{"hello":{"api":[7,9]}}"#);

    // Assert
    let refused = bar.recv().unwrap();
    assert_eq!(refused["error"]["code"], "unsupported-api");
    assert_eq!(refused["error"]["api"], json!([1]));
    assert!(bar.ended());
}

#[test]
fn a_session_must_start_with_hello() {
    // Arrange
    let world = World::new();
    let mut bar = world.listener.connect();

    // Act
    bar.send(r#"{"id":1,"ping":{}}"#);

    // Assert
    assert_eq!(bar.recv().unwrap()["error"]["code"], "malformed");
    assert!(bar.ended());
}

#[test]
fn a_bar_that_sends_no_hello_in_time_is_dropped() {
    // Arrange
    let limits = Limits {
        hello: Duration::from_millis(100),
        ..Limits::default()
    };
    let world = World::with(limits);

    // Act
    let mut bar = world.listener.connect();

    // Assert
    assert!(bar.ended());
    assert!(world.hub.wait_ended(Duration::from_secs(2)));
}

#[test]
fn hello_past_the_session_limit_is_answered_busy() {
    // Arrange
    let limits = Limits {
        sessions: 1,
        ..Limits::default()
    };
    let world = World::with(limits);
    world.hub.publish(&state(1));
    let _first = world.open(HELLO);
    let mut second = world.listener.connect();

    // Act
    second.send(HELLO);

    // Assert
    assert_eq!(second.recv().unwrap()["error"]["code"], "busy");
    assert!(second.ended());
}

#[test]
fn connections_past_the_limit_are_closed_unread() {
    // Arrange
    let limits = Limits {
        connections: 1,
        ..Limits::default()
    };
    let world = World::with(limits);
    let _first = world.listener.connect();

    // Act
    let mut second = world.listener.connect();

    // Assert
    assert!(second.ended());
}

#[test]
fn a_session_waits_for_replies_before_handing_the_loop_more_requests() {
    // Arrange
    let limits = Limits {
        in_flight: 1,
        ..Limits::default()
    };
    let world = World::with(limits);
    world.hub.publish(&state(1));
    let (mut bar, _, _) = world.open(HELLO);
    bar.send(r#"{"id":1,"run":{"action":"retile"}}"#);
    bar.send(r#"{"id":2,"run":{"action":"retile"}}"#);
    let LoopRequest::Run { reply: first, .. } = world.request() else {
        panic!("a run");
    };
    let held = world.quiet();

    // Act
    first.ok();

    // Assert
    assert!(held, "the second waits for the first's reply");
    assert!(matches!(world.request(), LoopRequest::Run { .. }));
    assert_eq!(bar.recv(), Some(json!({"id":1,"ok":{"rev":1}})));
}

#[test]
fn a_bar_that_leaves_ends_its_session_and_the_next_one_asks_for_the_state_again() {
    // Arrange
    let world = World::new();
    world.hub.publish(&state(1));
    let (bar, _, _) = world.open(HELLO);

    // Act
    bar.leave();
    let ended = world.sessions_become(0);
    world.listener.connect().send(HELLO);

    // Assert
    assert!(ended);
    assert!(matches!(world.request(), LoopRequest::Publish));
}

#[test]
fn a_message_too_large_ends_the_session() {
    // Arrange
    let world = World::new();
    world.hub.publish(&state(1));
    let (mut bar, _, _) = world.open(HELLO);

    // Act
    bar.send(&"x".repeat(MAX_MESSAGE_BYTES + 1));

    // Assert
    assert_eq!(bar.recv().unwrap()["error"]["code"], "too-large");
    assert!(bar.ended());
}

#[test]
fn a_bar_that_stops_reading_is_dropped_while_others_keep_getting_updates() {
    // Arrange
    let limits = Limits {
        write: Duration::from_millis(200),
        ..Limits::default()
    };
    let world = World::with(limits);
    world.hub.publish(&state(1));
    let stuck = world.listener.connect_from(2048, None);
    stuck.send(HELLO);
    let (mut reading, _, _) = world.open(HELLO);

    // Act
    let mut updates = Vec::new();
    for n in 0..20 {
        let mut next = state(1);
        next.daemon.update = Some(format!("{n} {}", "x".repeat(500)));
        world.hub.publish(&next);
        updates.push(reading.recv().unwrap());
    }

    // Assert
    let seqs: Vec<_> = updates.iter().map(|u| u["seq"].as_u64().unwrap()).collect();
    assert_eq!(seqs, (2..22).collect::<Vec<_>>(), "no update was missed");
    assert!(world.sessions_become(1), "the stuck bar's session ended");
}
