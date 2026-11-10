//! The space `subscribe --reserve` holds, and what a bar reserves through
//! it, sent again after a switch of provider.

use std::sync::Arc;
use std::thread::{self, JoinHandle};

use serde_json::{Value, json};

use crate::StableMonitorId;
use crate::api::{LoopRequest, MonitorTarget, Peer, Reserve, State};
use crate::system::ParentWatch;
use crate::tiler::Side;

use super::{Hold, MonitorRef, Outcome, Streams, Subscription, subscribe};
use crate::api::fake_client::{FakeApiClient, FakeParent, Printed, input};
use crate::api::session_world::{World, state};

fn start(
    world: &World,
    sub: Subscription,
    printed: &Printed,
    parent: Option<Arc<dyn ParentWatch>>,
) -> (JoinHandle<Outcome>, std::sync::mpsc::Sender<String>) {
    let client = FakeApiClient::to(&world.listener);
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

/// The next reservation asked of the loop, answered.
fn reserved(world: &World) -> (Peer, Reserve) {
    loop {
        if let LoopRequest::Reserve {
            peer,
            reserve,
            reply,
        } = world.request()
        {
            reply.ok();
            return (peer, reserve);
        }
    }
}

fn with_provider(provider: &str) -> State {
    let mut state = state(1);
    state.daemon.provider = provider.to_string();
    state
}

#[test]
fn a_hold_reads_an_edge_its_pixels_and_a_monitor() {
    // Act
    let held: Vec<Result<Hold, String>> =
        ["top=32", "Left=8@m-41c6e2", "bottom=4@1", "up=3", "top"]
            .iter()
            .map(|text| text.parse())
            .collect();

    // Assert
    let hold = |side, pixels, monitor| {
        Ok(Hold {
            side,
            pixels,
            monitor,
        })
    };
    assert_eq!(held[0], hold(Side::Top, 32, None));
    let id = MonitorRef::Id("m-41c6e2".to_string());
    assert_eq!(held[1], hold(Side::Left, 8, Some(id)));
    assert_eq!(held[2], hold(Side::Bottom, 4, Some(MonitorRef::Index(1))));
    assert!(held[3].is_err() && held[4].is_err());
}

#[test]
fn subscribe_reserves_its_space_for_the_bar_that_started_it_without_printing_the_answer() {
    // Arrange
    let world = World::new();
    world.hub.publish(&state(1));
    let printed = Printed::default();
    let parent = Arc::new(FakeParent::default());
    let sub = Subscription {
        bar: Some("yasb".to_string()),
        reserve: vec!["top=32".parse().unwrap(), "bottom=8".parse().unwrap()],
        ..Subscription::default()
    };
    let (running, _feed) = start(&world, sub, &printed, Some(parent.clone()));

    // Act
    let (peer, reserve) = reserved(&world);
    world.hub.publish(&state(2));
    printed.at_least(3);
    parent.exit();
    let outcome = running.join().unwrap();

    // Assert
    assert!(matches!(outcome, Outcome::Done), "{outcome:?}");
    assert_eq!(
        (peer.bar.as_deref(), peer.process),
        (Some("yasb"), Some(4242))
    );
    let expected = Reserve {
        monitor: MonitorTarget::All,
        top: 32,
        bottom: 8,
        ..Reserve::default()
    };
    assert_eq!(reserve, expected);
    let lines: Vec<Value> = (printed.lines().iter())
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert!(
        lines.iter().all(|line| line.get("id").is_none()),
        "{lines:?}"
    );
}

#[test]
fn a_bar_reserving_keeps_the_holds_on_its_monitor_and_a_switch_of_provider_sends_both_again() {
    // Arrange
    let world = World::new();
    world.hub.publish(&with_provider("external"));
    let printed = Printed::default();
    let sub = Subscription {
        monitor: Some(MonitorRef::Index(0)),
        requests: true,
        reserve: vec!["top=32".parse().unwrap()],
        ..Subscription::default()
    };
    let (running, feed) = start(&world, sub, &printed, None);
    let (_, first) = reserved(&world);
    feed.send(r#"{"id":1,"reserve":{"monitor":"m-1","bottom":20}}"#.to_string())
        .unwrap();
    let (_, merged) = reserved(&world);
    printed.at_least(3);

    // Act
    world.hub.publish(&with_provider("yasb"));
    let (_, again) = reserved(&world);
    drop(feed);
    let outcome = running.join().unwrap();

    // Assert
    assert!(matches!(outcome, Outcome::Done), "{outcome:?}");
    let on = |top, bottom| Reserve {
        monitor: MonitorTarget::One(StableMonitorId::new("m-1")),
        top,
        bottom,
        ..Reserve::default()
    };
    assert_eq!(first, on(32, 0), "named by the index --monitor gives");
    assert_eq!((merged, again), (on(32, 20), on(32, 20)));
    let answered: Vec<(Value, bool)> = (printed.lines().iter())
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(|line| line.get("id").is_some())
        .map(|line| (line["id"].clone(), line.get("ok").is_some()))
        .collect();
    assert_eq!(answered, [(json!(1), true)]);
}

#[test]
fn a_reserve_the_bar_sends_before_the_snapshot_is_not_undone_by_the_holds() {
    // Arrange -- the bar's line is read before the session's first
    // snapshot, so it waits for it.
    let world = World::new();
    let printed = Printed::default();
    let (feed, input) = input();
    feed.send(r#"{"id":1,"reserve":{"monitor":"m-1","bottom":20}}"#.to_string())
        .unwrap();
    let client = FakeApiClient::to(&world.listener);
    let sub = Subscription {
        monitor: Some(MonitorRef::Index(0)),
        requests: true,
        reserve: vec!["top=32".parse().unwrap()],
        ..Subscription::default()
    };
    let mut out = printed.clone();
    let running = thread::spawn(move || {
        let io = Streams {
            input,
            out: &mut out,
            parent: None,
        };
        subscribe(&client, "test", &sub, io)
    });
    assert!(world.sessions_become(1));
    thread::sleep(std::time::Duration::from_millis(100));

    // Act
    world.hub.publish(&with_provider("external"));
    let (_, first) = reserved(&world);
    let (_, last) = reserved(&world);
    printed.at_least(2);
    drop(feed);
    let outcome = running.join().unwrap();

    // Assert -- the daemon keeps the last reservation on a monitor.
    assert!(matches!(outcome, Outcome::Done), "{outcome:?}");
    let expected = Reserve {
        monitor: MonitorTarget::One(StableMonitorId::new("m-1")),
        top: 32,
        bottom: 20,
        ..Reserve::default()
    };
    assert_eq!(first, expected);
    assert_eq!(last, expected, "the bottom the bar asked for is kept");
}
