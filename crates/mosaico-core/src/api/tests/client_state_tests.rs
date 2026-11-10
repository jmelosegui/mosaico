use serde_json::{Value, json};

use super::output::Printer;
use super::{FieldPath, JsonState, MonitorRef, Output, field_text};

fn path(text: &str) -> FieldPath {
    text.parse().unwrap()
}

/// Each line read back as JSON, so key order does not matter.
fn parsed<const N: usize>(lines: [Option<String>; N]) -> [Option<Value>; N] {
    lines.map(|line| line.map(|line| serde_json::from_str(&line).unwrap()))
}

fn two_monitors() -> Value {
    json!({
        "daemon": {"version": "1.2.3"},
        "monitors": [
            {"id": "m-a", "index": 0, "active_workspace": 1},
            {"id": "m-b", "index": 1, "active_workspace": 4},
        ],
    })
}

#[test]
fn a_path_reaches_into_objects_and_lists() {
    // Arrange
    let state = two_monitors();

    // Act
    let found = path("monitors.1.active_workspace").lookup(&state);

    // Assert
    assert_eq!(found, Some(&json!(4)));
    assert_eq!(path("monitors.7.id").lookup(&state), None);
    assert_eq!(path("daemon.version.major").lookup(&state), None);
}

#[test]
fn a_path_with_an_empty_step_is_refused() {
    // Arrange, Act
    let parsed = ["", "focus.", ".focus", "monitors..id"].map(str::parse::<FieldPath>);

    // Assert
    assert!(parsed.iter().all(Result::is_err), "{parsed:?}");
    assert_eq!(path("focus.window.title").topic(), "focus");
}

#[test]
fn a_field_prints_as_one_line_of_text() {
    // Arrange
    let title = json!("C:\\temp\\a\nb\r");

    // Act
    let texts = [
        field_text(Some(&title)),
        field_text(Some(&json!(3))),
        field_text(Some(&json!(true))),
        field_text(Some(&Value::Null)),
        field_text(None),
        field_text(Some(&json!({"a": [1, "x\\y\n"]}))),
    ];

    // Assert
    let expected = [
        "C:\\\\temp\\\\a\\nb\\r",
        "3",
        "true",
        "",
        "",
        r#"{"a":[1,"x\\y\n"]}"#,
    ];
    assert_eq!(texts, expected);
}

#[test]
fn a_monitor_is_named_by_index_when_all_digits_and_by_id_otherwise() {
    // Arrange, Act
    let named = ["0", "12", "m-41c6e2", "1a"].map(|text| text.parse::<MonitorRef>().unwrap());

    // Assert
    let expected = [
        MonitorRef::Index(0),
        MonitorRef::Index(12),
        MonitorRef::Id("m-41c6e2".to_string()),
        MonitorRef::Id("1a".to_string()),
    ];
    assert_eq!(named, expected);
    assert!("".parse::<MonitorRef>().is_err());
}

#[test]
fn updates_replace_topics_and_monitors_by_id_and_keep_unknown_fields() {
    // Arrange
    let mut state = JsonState::default();
    state.reset(&json!({"monitors": [{"id": "m-a", "index": 0}], "future": 1}));

    // Act
    state.apply(
        &json!({"monitors": [{"id": "m-c", "index": 0}, {"id": "m-a", "index": 1, "new": true}]}),
    );
    state.apply(&json!({"daemon": {"paused": true}, "removed": {"monitors": ["m-c"]}}));

    // Assert
    let expected = json!({
        "monitors": [{"id": "m-a", "index": 1, "new": true}],
        "future": 1,
        "daemon": {"paused": true},
    });
    assert_eq!(state.view(None, None), expected);
}

#[test]
fn a_view_of_one_monitor_keeps_the_rest_of_the_state() {
    // Arrange
    let mut state = JsonState::default();
    state.reset(&two_monitors());

    // Act
    let view = state.view(None, Some(&MonitorRef::Index(1)));

    // Assert
    assert_eq!(
        view["monitors"],
        json!([{"id": "m-b", "index": 1, "active_workspace": 4}])
    );
    assert_eq!(view["daemon"]["version"], "1.2.3");
    assert_eq!(
        state.monitor_id(&MonitorRef::Index(1)),
        Some("m-b".to_string())
    );
    assert_eq!(state.monitor_id(&MonitorRef::Index(5)), None);
}

#[test]
fn messages_for_one_monitor_drop_the_others_but_keep_empty_updates() {
    // Arrange
    let monitor = Some(MonitorRef::Id("m-a".to_string()));
    let mut printer = Printer::new(Output::Messages, monitor, false);
    printer.take(&json!({"seq": 1, "rev": 1, "snapshot": two_monitors()}));

    // Act
    let other = json!({"seq": 2, "rev": 2, "update": {"monitors": [{"id": "m-b", "index": 1}]}});
    let gone = json!({"seq": 3, "rev": 3, "update": {"removed": {"monitors": ["m-b", "m-a"]}}});
    let lines = parsed([printer.take(&other), printer.take(&gone)]);

    // Assert
    let expected = [
        Some(json!({"seq": 2, "rev": 2, "update": {"monitors": []}})),
        Some(json!({"seq": 3, "rev": 3, "update": {"removed": {"monitors": ["m-a"]}}})),
    ];
    assert_eq!(lines, expected);
}

#[test]
fn the_full_state_is_printed_only_when_what_is_shown_changes() {
    // Arrange
    let monitor = Some(MonitorRef::Index(0));
    let mut printer = Printer::new(Output::Full, monitor, false);
    let welcome = json!({"welcome": {"api": 1}});

    // Act
    let lines = parsed([
        printer.take(&welcome),
        printer.take(&json!({"seq": 1, "rev": 5, "snapshot": two_monitors()})),
        printer.take(
            &json!({"seq": 2, "rev": 6, "update": {"monitors": [{"id": "m-b", "index": 1}]}}),
        ),
        printer.take(
            &json!({"seq": 3, "rev": 7, "update": {"monitors": [{"id": "m-a", "index": 0}]}}),
        ),
        printer.take(&json!({"id": 1, "ok": {}})),
    ]);

    // Assert
    let daemon = json!({"version": "1.2.3"});
    let first = json!([{"id": "m-a", "index": 0, "active_workspace": 1}]);
    let later = json!([{"id": "m-a", "index": 0}]);
    let expected = [
        None,
        Some(json!({"rev": 5, "state": {"daemon": daemon, "monitors": first}})),
        None,
        Some(json!({"rev": 7, "state": {"daemon": daemon, "monitors": later}})),
        Some(json!({"id": 1, "ok": {}})),
    ];
    assert_eq!(lines, expected);
}

#[test]
fn the_full_state_shows_only_the_topics_asked_for() {
    // Arrange
    let mut printer = Printer::new(Output::Full, None, false);
    printer.follow(Some(vec!["monitors".to_string()]));
    let paused = json!({"seq": 2, "rev": 6, "update": {"daemon": {"paused": true}}});

    // Act
    let lines = parsed([
        printer.take(&json!({"seq": 1, "rev": 5, "snapshot": two_monitors()})),
        printer.take(&paused),
    ]);

    // Assert
    let monitors = two_monitors()["monitors"].clone();
    let expected = [
        Some(json!({"rev": 5, "state": {"monitors": monitors}})),
        None,
    ];
    assert_eq!(lines, expected);
}

#[test]
fn a_field_is_printed_when_it_changes_and_replies_only_when_requests_are_read() {
    // Arrange
    let field = Output::Field(path("monitors.0.active_workspace"));
    let mut quiet = Printer::new(field.clone(), None, false);
    let mut requests = Printer::new(field, None, true);
    let snapshot = json!({"seq": 1, "rev": 1, "snapshot": two_monitors()});
    let same = json!({"seq": 2, "rev": 2, "update": {"daemon": {"paused": true}}});
    let reply = json!({"id": 3, "ok": {}});

    // Act
    let lines = [quiet.take(&snapshot), quiet.take(&same), quiet.take(&reply)];
    let replied = [requests.take(&snapshot), requests.take(&reply)];

    // Assert
    assert_eq!(lines, [Some("1".to_string()), None, None]);
    assert_eq!(parsed(replied), [Some(json!(1)), Some(reply)]);
}
