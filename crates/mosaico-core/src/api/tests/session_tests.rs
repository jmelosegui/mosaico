use serde_json::json;

use crate::StableMonitorId;
use crate::api::LoopRequest;

use super::session_world::{HELLO, World, keys, state};

#[test]
fn a_bar_is_welcomed_and_sent_the_state_once_the_loop_publishes_it() {
    // Arrange
    let world = World::new();
    let mut bar = world.listener.connect();

    // Act
    bar.send(HELLO);
    let asked = world.request();
    let before = bar.recv_within(std::time::Duration::from_millis(100));
    world.hub.publish(&state(1));

    // Assert
    assert!(matches!(asked, LoopRequest::Publish));
    assert_eq!(before, None, "nothing is sent before the state is");
    let welcome = json!({"welcome":{"api":1,"mosaico":"1.2.3","boot":"b00t","provider":"external",
        "topics":["daemon","theme","monitors","focus"]}});
    assert_eq!(bar.recv(), Some(welcome));
    let snapshot = bar.recv().unwrap();
    assert_eq!((&snapshot["seq"], &snapshot["rev"]), (&json!(1), &json!(1)));
    assert_eq!(
        keys(&snapshot, "snapshot"),
        ["daemon", "focus", "monitors", "theme"]
    );
    assert_eq!(snapshot["snapshot"]["monitors"][0]["active_workspace"], 1);
}

#[test]
fn an_update_carries_only_what_changed() {
    // Arrange
    let world = World::new();
    world.hub.publish(&state(1));
    let (mut bar, _, _) = world.open(HELLO);

    // Act
    world.hub.publish(&state(2));

    // Assert
    let update = bar.recv().unwrap();
    assert_eq!((&update["seq"], &update["rev"]), (&json!(2), &json!(2)));
    assert_eq!(keys(&update, "update"), ["focus", "monitors"]);
    assert_eq!(update["update"]["focus"]["workspace"], 2);
}

#[test]
fn a_session_follows_the_daemon_and_the_topics_it_asked_for_that_exist() {
    // Arrange
    let world = World::new();
    world.hub.publish(&state(1));
    let hello = r#"{"hello":{"api":[1],"topics":["focus","weather"]}}"#;
    let (mut bar, welcome, snapshot) = world.open(hello);
    let mut renamed = state(1);
    renamed.monitors[0].name = "DELL".to_string();

    // Act
    world.hub.publish(&renamed);
    let unfollowed = bar.recv_within(std::time::Duration::from_millis(150));
    world.hub.publish(&state(2));

    // Assert
    assert_eq!(welcome["welcome"]["topics"], json!(["daemon", "focus"]));
    assert_eq!(keys(&snapshot, "snapshot"), ["daemon", "focus"]);
    assert_eq!(unfollowed, None, "a monitor change is not followed");
    let update = bar.recv().unwrap();
    assert_eq!(keys(&update, "update"), ["focus"]);
    assert_eq!(update["seq"], 2);
}

#[test]
fn subscribe_and_resync_are_answered_with_a_snapshot_carrying_their_id() {
    // Arrange
    let world = World::new();
    world.hub.publish(&state(1));
    let (mut bar, _, _) = world.open(HELLO);

    // Act
    bar.send(r#"{"id":7,"subscribe":{"topics":["monitors"]}}"#);
    let subscribed = bar.recv().unwrap();
    bar.send(r#"{"id":8,"resync":{}}"#);
    let resynced = bar.recv().unwrap();
    bar.send(r#"{"id":9,"ping":{}}"#);

    // Assert
    assert_eq!(
        (&subscribed["id"], &subscribed["seq"]),
        (&json!(7), &json!(2))
    );
    assert_eq!(keys(&subscribed, "snapshot"), ["daemon", "monitors"]);
    assert_eq!((&resynced["id"], &resynced["seq"]), (&json!(8), &json!(3)));
    assert_eq!(keys(&resynced, "snapshot"), ["daemon", "monitors"]);
    assert_eq!(bar.recv(), Some(json!({"id":9,"ok":{}})));
}

#[test]
fn a_run_goes_to_the_loop_and_its_reply_comes_after_the_state_it_changed() {
    // Arrange
    let world = World::new();
    world.hub.publish(&state(1));
    let (mut bar, _, _) = world.open(HELLO);

    // Act
    bar.send(r#"{"id":3,"run":{"action":"goto-workspace-2","monitor":"m-1"}}"#);
    let LoopRequest::Run {
        action,
        monitor,
        reply,
    } = world.request()
    else {
        panic!("a run");
    };
    world.hub.publish(&state(2));
    reply.ok();

    // Assert
    assert_eq!(action.to_string(), "goto-workspace-2");
    assert_eq!(monitor, Some(StableMonitorId::new("m-1")));
    let update = bar.recv().unwrap();
    assert_eq!(
        (&update["rev"], &update["update"]["focus"]["workspace"]),
        (&json!(2), &json!(2))
    );
    assert_eq!(bar.recv(), Some(json!({"id":3,"ok":{"rev":2}})));
}

#[test]
fn a_bar_may_not_run_what_acts_on_the_focused_window_nor_what_does_not_exist() {
    // Arrange
    let world = World::new();
    world.hub.publish(&state(1));
    let (mut bar, _, _) = world.open(HELLO);

    // Act
    bar.send(r#"{"id":1,"run":{"action":"close-focused"}}"#);
    bar.send(r#"{"id":2,"run":{"action":"fly"}}"#);

    // Assert
    assert_eq!(bar.recv().unwrap()["error"]["code"], "not-allowed");
    let unknown = bar.recv().unwrap();
    assert_eq!(
        (&unknown["id"], &unknown["error"]["code"]),
        (&json!(2), &json!("unknown-action"))
    );
    assert!(world.quiet(), "the loop is asked nothing");
}

#[test]
fn a_request_it_cannot_read_is_refused_and_the_session_goes_on() {
    // Arrange
    let world = World::new();
    world.hub.publish(&state(1));
    let (mut bar, _, _) = world.open(HELLO);

    // Act
    bar.send(r#"{"id":4,"teleport":{}}"#);
    bar.send("not json");
    bar.send(r#"{"id":5,"run":{}}"#);
    bar.send(r#"{"id":6,"ping":{}}"#);

    // Assert
    let unknown = bar.recv().unwrap();
    assert_eq!(
        (&unknown["id"], &unknown["error"]["code"]),
        (&json!(4), &json!("unknown-request"))
    );
    let garbled = bar.recv().unwrap();
    assert_eq!(
        (garbled.get("id"), &garbled["error"]["code"]),
        (None, &json!("malformed"))
    );
    let bad_run = bar.recv().unwrap();
    assert_eq!(
        (&bad_run["id"], &bad_run["error"]["code"]),
        (&json!(5), &json!("malformed"))
    );
    assert_eq!(bar.recv(), Some(json!({"id":6,"ok":{}})));
}

#[test]
fn a_run_the_loop_drops_is_answered_stopping() {
    // Arrange
    let world = World::new();
    world.hub.publish(&state(1));
    let (mut bar, _, _) = world.open(HELLO);
    bar.send(r#"{"id":1,"run":{"action":"retile"}}"#);

    // Act
    drop(world.request());

    // Assert
    let refused = bar.recv().unwrap();
    assert_eq!(
        (&refused["id"], &refused["error"]["code"]),
        (&json!(1), &json!("stopping"))
    );
}

#[test]
fn closing_says_bye_to_every_session_and_ends_them() {
    // Arrange
    let world = World::new();
    world.hub.publish(&state(1));
    let (mut first, _, _) = world.open(HELLO);
    let (mut second, _, _) = world.open(HELLO);

    // Act
    world.hub.close("stopping");

    // Assert
    let bye = json!({"bye":{"reason":"stopping"}});
    assert_eq!(first.recv(), Some(bye.clone()));
    assert_eq!(second.recv(), Some(bye));
    assert!(first.ended() && second.ended());
    assert!(world.hub.wait_ended(std::time::Duration::from_secs(2)));
}
