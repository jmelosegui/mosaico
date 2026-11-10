//! The session's messages: how a bar's are read, and the examples and
//! schemas under `docs/bar-api` that pin their shape. Bless the files as
//! `examples_tests.rs` says.

use serde::Serialize;
use serde_json::json;

use crate::api::{
    Ack, ApiError, Bye, ClientMessage, Daemon, DaemonMessage, ErrorCode, Focus, FocusedWindow,
    Hello, Request, Run, Snapshot, Subscribe, Update, Welcome,
};
use crate::{StableMonitorId, WindowId};

use super::examples_tests::{check, path, pretty};

fn client_examples() -> Vec<ClientMessage> {
    let request = |id, request| ClientMessage::Request { id, request };
    vec![
        ClientMessage::Hello {
            hello: Hello {
                api: vec![1],
                bar: Some("yasb".to_string()),
                client: Some("mosaico-yasb 1.0".to_string()),
                topics: Some(vec!["daemon".to_string(), "focus".to_string()]),
            },
        },
        request(
            1,
            Request::Run(Run {
                action: "goto-workspace-3".to_string(),
                monitor: Some(StableMonitorId::new("m-41c6e2")),
            }),
        ),
        request(
            2,
            Request::Subscribe(Subscribe {
                topics: vec!["monitors".to_string(), "focus".to_string()],
            }),
        ),
        request(3, Request::Resync {}),
        request(4, Request::Ping {}),
    ]
}

fn daemon_examples() -> Vec<DaemonMessage> {
    let focus = |workspace, title: &str| Focus {
        monitor: Some(StableMonitorId::new("m-41c6e2")),
        workspace: Some(workspace),
        window: Some(FocusedWindow {
            id: WindowId::from_raw(132_456),
            title: title.to_string(),
            program: "firefox.exe".to_string(),
        }),
    };
    let daemon = Daemon {
        version: "0.15.0".to_string(),
        api: 1,
        provider: "yasb".to_string(),
        workspace_mode: "per-monitor".to_string(),
        ..Daemon::default()
    };
    let refused = ApiError {
        api: Some(vec![1]),
        ..ApiError::new(ErrorCode::UnsupportedApi, "no API version in common")
    };
    vec![
        DaemonMessage::Welcome {
            welcome: Welcome {
                api: 1,
                mosaico: "0.15.0".to_string(),
                boot: "5f3c9a1e".to_string(),
                provider: "yasb".to_string(),
                topics: vec!["daemon".to_string(), "focus".to_string()],
            },
        },
        DaemonMessage::Snapshot {
            id: None,
            seq: 1,
            rev: 57,
            snapshot: Snapshot {
                daemon: Some(daemon),
                focus: Some(focus(2, "Mozilla Firefox")),
                ..Snapshot::default()
            },
        },
        DaemonMessage::Update {
            seq: 2,
            rev: 59,
            update: Update {
                focus: Some(focus(3, "mosaico - Mozilla Firefox")),
                ..Update::default()
            },
        },
        DaemonMessage::Ok {
            id: 1,
            ok: Ack { rev: Some(59) },
        },
        DaemonMessage::Ok {
            id: 4,
            ok: Ack::default(),
        },
        DaemonMessage::Error {
            id: Some(5),
            error: ApiError::new(
                ErrorCode::NotAllowed,
                "`close-focused` acts on the focused window, which a bar may not",
            ),
        },
        DaemonMessage::Error {
            id: None,
            error: refused,
        },
        DaemonMessage::Bye {
            bye: Bye {
                reason: "stopping".to_string(),
            },
        },
    ]
}

fn lines(messages: &[impl Serialize]) -> String {
    let line = |m| serde_json::to_string(m).unwrap_or_default() + "\n";
    messages.iter().map(line).collect()
}

/// Reads each line of the example `name` and writes it back with
/// `again`.
fn round_trip(name: &str, again: impl Fn(&str) -> String) {
    // Arrange
    let text = std::fs::read_to_string(path(name)).unwrap_or_default();

    // Act
    let written: String = (text.lines()).map(|line| again(line) + "\n").collect();

    // Assert
    assert_eq!(
        written,
        text.replace("\r\n", "\n"),
        "{name} does not write back the same"
    );
}

#[test]
fn the_message_examples_are_up_to_date() {
    check("client.jsonl", &lines(&client_examples()));
    check("daemon.jsonl", &lines(&daemon_examples()));
}

#[test]
fn the_bar_messages_read_and_write_back_the_same() {
    round_trip("client.jsonl", |line| {
        let message = ClientMessage::parse(serde_json::from_str(line).unwrap()).unwrap();
        serde_json::to_string(&message).unwrap()
    });
}

#[test]
fn the_daemon_messages_read_and_write_back_the_same() {
    round_trip("daemon.jsonl", |line| {
        let message: DaemonMessage = serde_json::from_str(line).unwrap();
        serde_json::to_string(&message).unwrap()
    });
}

#[test]
fn the_message_schemas_are_up_to_date() {
    let client = schemars::schema_for!(ClientMessage);
    check("client-message.schema.json", &pretty(&client));
    let daemon = schemars::schema_for!(DaemonMessage);
    check("daemon-message.schema.json", &pretty(&daemon));
}

#[test]
fn a_request_of_an_unknown_kind_is_refused_with_its_id() {
    // Act
    let refused = ClientMessage::parse(json!({"id": 4, "teleport": {}}));

    // Assert
    let refusal = refused.unwrap_err();
    assert_eq!(
        (refusal.id, refusal.error.code),
        (Some(4), ErrorCode::UnknownRequest)
    );
}

#[test]
fn a_request_without_a_whole_number_id_is_malformed() {
    // Act
    let refused = ClientMessage::parse(json!({"id": "one", "ping": {}}));

    // Assert
    let refusal = refused.unwrap_err();
    assert_eq!(
        (refusal.id, refusal.error.code),
        (None, ErrorCode::Malformed)
    );
}

#[test]
fn a_bar_reads_an_error_code_it_does_not_know() {
    // Act
    let read: DaemonMessage =
        serde_json::from_value(json!({"error": {"code": "later", "message": "?"}})).unwrap();

    // Assert
    let DaemonMessage::Error { error, .. } = read else {
        panic!("an error");
    };
    assert_eq!(error.code, ErrorCode::Other);
}
