use std::io::{self, Write};
use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use crate::StableMonitorId;
use crate::api::LoopRequest;
use crate::system::ParentWatch;

use super::{ApiClient, MonitorRef, Outcome, Output, Streams, Subscription, subscribe};
use crate::api::fake_client::{FakeApiClient, FakeParent, Printed, Scripted, input};
use crate::api::session_world::{World, state};

/// Runs `subscribe` on a thread of its own, with `feed` feeding its input
/// when given; without one the input never ends.
fn start(
    client: impl ApiClient + Send + 'static,
    sub: Subscription,
    printed: &Printed,
    parent: Option<Arc<dyn ParentWatch>>,
) -> (JoinHandle<Outcome>, Sender<String>) {
    let (feed, input) = input();
    let mut out = printed.clone();
    let running = thread::spawn(move || {
        let io = Streams {
            input,
            out: &mut out,
            parent,
        };
        subscribe(&client, "test", &sub, io)
    });
    (running, feed)
}

fn json_lines(lines: &[String]) -> Vec<Value> {
    lines
        .iter()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

#[test]
fn subscribe_prints_each_message_until_the_daemon_says_bye() {
    // Arrange
    let world = World::new();
    world.hub.publish(&state(1));
    let client = FakeApiClient::to(&world.listener);
    let printed = Printed::default();
    let (running, _feed) = start(client, Subscription::default(), &printed, None);
    printed.at_least(2);

    // Act
    world.hub.publish(&state(2));
    printed.at_least(3);
    drop(world);
    let outcome = running.join().unwrap();

    // Assert
    assert!(matches!(outcome, Outcome::Done), "{outcome:?}");
    let lines = json_lines(&printed.lines());
    let kinds = ["welcome", "snapshot", "update", "bye"];
    let kind = |line: &Value| kinds.into_iter().find(|kind| line.get(kind).is_some());
    let found: Vec<_> = lines.iter().map(kind).collect();
    assert_eq!(found, kinds.map(Some));
    assert_eq!(lines[2]["update"]["focus"]["workspace"], 2);
}

#[test]
fn subscribe_without_wait_fails_when_the_daemon_is_lost_or_not_running() {
    // Arrange
    let printed = Printed::default();
    let lost = Scripted("{\"welcome\":{\"api\":1}}\n");

    // Act
    let outcomes = [
        start(lost, Subscription::default(), &printed, None).0,
        start(
            FakeApiClient::default(),
            Subscription::default(),
            &printed,
            None,
        )
        .0,
    ]
    .map(|running| running.join().unwrap());

    // Assert
    assert!(matches!(outcomes[0], Outcome::Lost), "{:?}", outcomes[0]);
    assert!(
        matches!(outcomes[1], Outcome::NotRunning),
        "{:?}",
        outcomes[1]
    );
}

#[test]
fn a_field_is_printed_as_it_changes_until_the_parent_exits() {
    // Arrange
    let world = World::new();
    world.hub.publish(&state(1));
    let client = FakeApiClient::to(&world.listener);
    let printed = Printed::default();
    let parent = Arc::new(FakeParent::default());
    let sub = Subscription {
        output: Output::Field("focus.workspace".parse().unwrap()),
        ..Subscription::default()
    };
    let (running, _feed) = start(client, sub, &printed, Some(parent.clone()));
    printed.at_least(1);

    // Act
    let mut renamed = state(2);
    renamed.monitors[0].name = "DELL".to_string();
    world.hub.publish(&state(2));
    printed.at_least(2);
    world.hub.publish(&renamed);
    world.hub.publish(&state(3));
    printed.at_least(3);
    parent.exit();
    let outcome = running.join().unwrap();

    // Assert
    assert!(matches!(outcome, Outcome::Done), "{outcome:?}");
    assert_eq!(printed.lines(), ["1", "2", "3"]);
}

#[test]
fn requests_are_forwarded_on_the_monitor_named_and_the_input_going_quiet_ends_it() {
    // Arrange
    let world = World::new();
    world.hub.publish(&state(1));
    let client = FakeApiClient::to(&world.listener);
    let printed = Printed::default();
    let sub = Subscription {
        output: Output::Field("focus.workspace".parse().unwrap()),
        monitor: Some(MonitorRef::Index(0)),
        requests: true,
        idle: Duration::from_millis(300),
        ..Subscription::default()
    };
    let (running, feed) = start(client, sub, &printed, None);
    printed.at_least(1);

    // Act
    feed.send("not json".to_string()).unwrap();
    feed.send(r#"{"id":4,"run":{"action":"goto-workspace-2"}}"#.to_string())
        .unwrap();
    let run = loop {
        if let LoopRequest::Run { monitor, reply, .. } = world.request() {
            break (monitor, reply);
        }
    };
    run.1.ok();
    let started = Instant::now();
    let outcome = running.join().unwrap();

    // Assert
    assert_eq!(run.0, Some(StableMonitorId::new("m-1")));
    assert!(matches!(outcome, Outcome::Done), "{outcome:?}");
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "it ends once quiet"
    );
    let lines = printed.lines();
    assert_eq!(lines[0], "1");
    let replies = json_lines(&lines[1..]);
    let malformed = json!({"error": {"code": "malformed", "message": "not JSON"}});
    assert_eq!(replies[0], malformed);
    assert_eq!(
        (&replies[1]["id"], replies[1].get("ok").is_some()),
        (&json!(4), true)
    );
}

#[test]
fn piped_requests_wait_for_the_snapshot_and_the_input_ending_waits_for_their_answers() {
    // Arrange
    let world = World::new();
    world.hub.publish(&state(1));
    let client = FakeApiClient::to(&world.listener);
    let printed = Printed::default();
    let sub = Subscription {
        monitor: Some(MonitorRef::Index(0)),
        requests: true,
        ..Subscription::default()
    };
    let (running, feed) = start(client, sub, &printed, None);

    // Act
    feed.send(r#"{"id":4,"run":{"action":"goto-workspace-2"}}"#.to_string())
        .unwrap();
    drop(feed);
    let run = loop {
        if let LoopRequest::Run { monitor, reply, .. } = world.request() {
            break (monitor, reply);
        }
    };
    run.1.ok();
    let outcome = running.join().unwrap();

    // Assert
    assert_eq!(run.0, Some(StableMonitorId::new("m-1")), "named by index");
    assert!(matches!(outcome, Outcome::Done), "{outcome:?}");
    let lines = json_lines(&printed.lines());
    let last = lines.last().unwrap();
    assert_eq!((&last["id"], last.get("ok").is_some()), (&json!(4), true));
}

#[test]
fn with_wait_a_restart_fails_what_was_in_flight_and_resumes_the_topics_followed() {
    // Arrange
    let first = World::new();
    first.hub.publish(&state(1));
    let client = FakeApiClient::to(&first.listener);
    let printed = Printed::default();
    let sub = Subscription {
        wait: true,
        requests: true,
        retry: Duration::from_millis(20),
        ..Subscription::default()
    };
    let (running, feed) = start(client.clone(), sub, &printed, None);
    printed.at_least(2);
    feed.send(r#"{"id":1,"subscribe":{"topics":["focus"]}}"#.to_string())
        .unwrap();
    printed.at_least(3);
    feed.send(r#"{"id":2,"run":{"action":"goto-workspace-2"}}"#.to_string())
        .unwrap();
    let held = loop {
        if let LoopRequest::Run { reply, .. } = first.request() {
            break reply;
        }
    };

    // Act
    client.listen(None);
    drop(first);
    drop(held);
    printed.at_least(6);
    feed.send(r#"{"id":3,"ping":{}}"#.to_string()).unwrap();
    printed.at_least(7);
    let second = World::new();
    second.hub.publish(&state(5));
    client.listen(Some(&second.listener));
    let lines = json_lines(&printed.at_least(9));
    drop(feed);
    let outcome = running.join().unwrap();

    // Assert
    assert!(matches!(outcome, Outcome::Done), "{outcome:?}");
    let disconnected = |id| {
        json!({"id": id, "error": {"code": "disconnected",
        "message": "the daemon went away before answering"}})
    };
    assert_eq!(lines[3], json!({"bye": {"reason": "test over"}}));
    assert_eq!(lines[4], json!({"disconnected": {"reason": "test over"}}));
    assert_eq!(lines[5..7], [disconnected(2), disconnected(3)]);
    assert_eq!(lines[7]["welcome"]["topics"], json!(["daemon", "focus"]));
    assert_eq!(lines[8]["snapshot"]["focus"]["workspace"], 5);
}

#[test]
fn a_failed_write_to_the_output_ends_it() {
    // Arrange
    struct Closed;
    impl Write for Closed {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::ErrorKind::BrokenPipe.into())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let world = World::new();
    world.hub.publish(&state(1));
    let client = FakeApiClient::to(&world.listener);
    let (_feed, input) = input();

    // Act
    let io = Streams {
        input,
        out: &mut Closed,
        parent: None,
    };
    let outcome = subscribe(&client, "test", &Subscription::default(), io);

    // Assert
    assert!(matches!(outcome, Outcome::Done), "{outcome:?}");
}
