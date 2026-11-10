use std::thread;

use serde_json::Value;

use crate::StableMonitorId;
use crate::api::{ErrorCode, LoopRequest, Reply};

use super::{ApiClient, MonitorRef, Outcome, Query, query, run};
use crate::api::fake_client::{FakeApiClient, Scripted};
use crate::api::session_world::{World, state};

fn queried(client: &dyn ApiClient, asked: &Query) -> (Outcome, String) {
    let mut out = Vec::new();
    let outcome = query(client, "test", asked, &mut out);
    (outcome, String::from_utf8(out).unwrap())
}

/// Publishes `state(workspace)` once no session is open, as the daemon
/// loop does when the next session asks for it.
fn idle_with(world: &World, workspace: u32) {
    assert!(world.sessions_become(0));
    world.hub.publish(&state(workspace));
}

/// The next run the loop is asked to carry out, past any publish.
fn next_run(world: &World) -> (String, Option<StableMonitorId>, Reply) {
    loop {
        if let LoopRequest::Run {
            action,
            monitor,
            reply,
        } = world.request()
        {
            return (action.to_string(), monitor, reply);
        }
    }
}

#[test]
fn query_prints_the_state_once_as_json() {
    // Arrange
    let world = World::new();
    world.hub.publish(&state(3));
    let client = FakeApiClient::to(&world.listener);

    // Act
    let (outcome, printed) = queried(&client, &Query::default());

    // Assert
    assert!(matches!(outcome, Outcome::Done), "{outcome:?}");
    let printed: Value = serde_json::from_str(&printed).unwrap();
    assert_eq!(printed, serde_json::to_value(state(3)).unwrap());
}

#[test]
fn query_of_topics_prints_only_those_though_the_session_follows_daemon_too() {
    // Arrange
    let world = World::new();
    world.hub.publish(&state(3));
    let client = FakeApiClient::to(&world.listener);
    let asked = Query {
        topics: Some(vec!["monitors".to_string()]),
        ..Query::default()
    };

    // Act
    let (outcome, printed) = queried(&client, &asked);

    // Assert
    assert!(matches!(outcome, Outcome::Done), "{outcome:?}");
    let printed: Value = serde_json::from_str(&printed).unwrap();
    let monitors = serde_json::to_value(state(3)).unwrap()["monitors"].clone();
    assert_eq!(printed, serde_json::json!({ "monitors": monitors }));
}

#[test]
fn query_of_a_field_prints_it_as_text_for_the_monitor_named() {
    // Arrange
    let world = World::new();
    world.hub.publish(&state(3));
    let client = FakeApiClient::to(&world.listener);
    let by_id = Query {
        monitor: Some(MonitorRef::Id("m-1".to_string())),
        field: Some("monitors.0.active_workspace".parse().unwrap()),
        ..Query::default()
    };
    let missing = Query {
        monitor: Some(MonitorRef::Index(4)),
        ..by_id.clone()
    };

    // Act
    let found = queried(&client, &by_id).1;
    idle_with(&world, 3);
    let not_found = queried(&client, &missing).1;

    // Assert
    assert_eq!(found, "3\n");
    assert_eq!(not_found, "\n");
}

#[test]
fn a_command_tells_apart_no_daemon_a_lost_one_and_one_of_another_version() {
    // Arrange
    let none = FakeApiClient::default();
    let mute = Scripted("");
    let other = Scripted(concat!(
        "not json\n",
        r#"{"error":{"code":"unsupported-api","message":"no","api":[2,3]}}"#,
        "\n"
    ));

    // Act
    let outcomes =
        [&none as &dyn ApiClient, &mute, &other].map(|c| queried(c, &Query::default()).0);

    // Assert
    assert!(
        matches!(outcomes[0], Outcome::NotRunning),
        "{:?}",
        outcomes[0]
    );
    assert!(matches!(outcomes[1], Outcome::Lost), "{:?}", outcomes[1]);
    assert!(
        matches!(&outcomes[2], Outcome::Mismatch(api) if *api == [2, 3]),
        "{:?}",
        outcomes[2]
    );
}

#[test]
fn run_carries_out_an_action_on_the_monitor_at_an_index() {
    // Arrange
    let world = World::new();
    world.hub.publish(&state(1));
    let client = FakeApiClient::to(&world.listener);

    // Act
    let running = thread::spawn(move || {
        run(
            &client,
            "test",
            "goto-workspace-2",
            Some(&MonitorRef::Index(0)),
        )
    });
    let (action, monitor, reply) = next_run(&world);
    reply.ok();
    let outcome = running.join().unwrap();

    // Assert
    assert_eq!(action, "goto-workspace-2");
    assert_eq!(monitor, Some(StableMonitorId::new("m-1")));
    assert!(matches!(outcome, Outcome::Done), "{outcome:?}");
}

#[test]
fn run_reports_what_the_daemon_refused() {
    // Arrange
    let world = World::new();
    world.hub.publish(&state(1));
    let client = FakeApiClient::to(&world.listener);
    let failing = client.clone();

    // Act
    let failed = thread::spawn(move || run(&failing, "test", "goto-workspace-2", None));
    let (_, monitor, reply) = next_run(&world);
    reply.fail(ErrorCode::Failed, "it broke");
    let failed = failed.join().unwrap();
    idle_with(&world, 1);
    let not_allowed = run(&client, "test", "close-focused", None);
    idle_with(&world, 1);
    let no_monitor = run(
        &client,
        "test",
        "goto-workspace-2",
        Some(&MonitorRef::Index(9)),
    );

    // Assert
    assert_eq!(monitor, None, "no monitor runs on the focused one");
    let code = |outcome: &Outcome| match outcome {
        Outcome::Refused(error) => Some(error.code),
        _ => None,
    };
    let codes = [&failed, &not_allowed, &no_monitor].map(code);
    let expected = [
        ErrorCode::Failed,
        ErrorCode::NotAllowed,
        ErrorCode::NoMonitor,
    ];
    assert_eq!(codes, expected.map(Some));
    assert!(matches!(&failed, Outcome::Refused(e) if e.message == "it broke"));
}
