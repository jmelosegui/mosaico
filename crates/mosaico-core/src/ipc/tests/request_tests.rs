use serde_json::json;

use super::*;
use crate::ipc::{ResponseStatus, read_message};
use crate::{Action, Direction};

fn refusal(value: Value) -> Response {
    decode_request(value).expect_err("request should be refused")
}

#[test]
fn a_request_is_the_command_plus_the_version() {
    // Arrange
    let request = Request::new(Command::Stop);

    // Act
    let wire = serde_json::to_string(&request).unwrap();

    // Assert
    assert_eq!(wire, r#"{"version":1,"command":"Stop"}"#);
}

#[test]
fn a_current_request_is_decoded() {
    // Arrange
    let action = Action::Focus(Direction::Left);
    let value = serde_json::to_value(Request::new(Command::Action { action })).unwrap();

    // Act
    let command = decode_request(value);

    // Assert
    assert_eq!(command, Ok(Command::Action { action }));
}

#[test]
fn a_daemon_without_versioning_still_reads_a_versioned_command() {
    // Arrange -- an older daemon deserializes straight into Command.
    let wire = serde_json::to_string(&Request::new(Command::Stop)).unwrap();

    // Act
    let command: Command = serde_json::from_str(&wire).unwrap();

    // Assert
    assert_eq!(command, Command::Stop);
}

#[test]
fn a_request_from_another_version_is_refused_with_the_daemon_version() {
    // Arrange
    let value = json!({ "version": PROTOCOL_VERSION + 1, "command": "Status" });

    // Act
    let response = refusal(value);

    // Assert
    assert_eq!(response.status, ResponseStatus::Error);
    assert_eq!(response.code, Some(ErrorCode::VersionMismatch));
    let status: DaemonStatus = response.data().expect("daemon status attached");
    assert_eq!(status.protocol, PROTOCOL_VERSION);
    assert_eq!(status.version, DAEMON_VERSION);
}

#[test]
fn an_unversioned_request_is_refused() {
    // Arrange -- what a CLI from before versioning sends.
    let value = json!({ "command": "Status" });

    // Act
    let response = refusal(value);

    // Assert
    assert_eq!(response.code, Some(ErrorCode::VersionMismatch));
}

#[test]
fn stop_is_honoured_from_any_version() {
    // Arrange
    let unversioned = json!({ "command": "Stop" });
    let newer = json!({ "version": PROTOCOL_VERSION + 1, "command": "Stop" });

    // Act / Assert
    assert_eq!(decode_request(unversioned), Ok(Command::Stop));
    assert_eq!(decode_request(newer), Ok(Command::Stop));
}

#[test]
fn an_unknown_command_gets_its_own_code() {
    // Arrange
    let value = json!({ "version": PROTOCOL_VERSION, "command": "Teleport" });

    // Act
    let response = refusal(value);

    // Assert
    assert_eq!(response.code, Some(ErrorCode::UnknownCommand));
    assert_eq!(
        response.message.as_deref(),
        Some("unknown command `Teleport`")
    );
}

/// The names serde accepts for a [`Command`], as it lists them when it
/// meets a name it does not know.
fn names_serde_knows() -> &'static [&'static str] {
    /// Keeps the names that `unknown_variant` reports, and nothing else.
    #[derive(Debug)]
    struct Expected(&'static [&'static str]);

    impl std::fmt::Display for Expected {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "expected one of {:?}", self.0)
        }
    }

    impl std::error::Error for Expected {}

    impl serde::de::Error for Expected {
        fn custom<T: std::fmt::Display>(_: T) -> Self {
            Self(&[])
        }

        fn unknown_variant(_: &str, expected: &'static [&'static str]) -> Self {
            Self(expected)
        }
    }

    let request = std::iter::once(("command", "not a command"));
    let probe = serde::de::value::MapDeserializer::<_, Expected>::new(request);
    Command::deserialize(probe).expect_err("not a command").0
}

#[test]
fn every_command_is_known_by_name() {
    // Act -- serde lists every variant, so a new command fails here until
    // it is added to COMMAND_NAMES.
    let names = names_serde_knows();

    // Assert
    assert_eq!(names, COMMAND_NAMES);
}

#[test]
fn a_command_with_missing_fields_is_malformed() {
    // Arrange
    let value = json!({ "version": PROTOCOL_VERSION, "command": "Action" });

    // Act
    let response = refusal(value);

    // Assert
    assert_eq!(response.code, Some(ErrorCode::Malformed));
}

#[test]
fn an_action_this_build_does_not_know_is_malformed() {
    // Arrange -- the command is known, only its field is not.
    let value = json!({ "version": PROTOCOL_VERSION, "command": "Action", "action": "teleport" });

    // Act
    let response = refusal(value);

    // Assert
    assert_eq!(response.code, Some(ErrorCode::Malformed));
}

#[test]
fn a_request_without_a_command_is_malformed() {
    // Arrange
    let missing = json!({ "version": PROTOCOL_VERSION });
    let not_a_name = json!({ "version": PROTOCOL_VERSION, "command": 7 });

    // Act / Assert
    assert_eq!(refusal(missing).code, Some(ErrorCode::Malformed));
    assert_eq!(refusal(not_a_name).code, Some(ErrorCode::Malformed));
}

#[test]
fn a_request_that_is_not_an_object_is_malformed() {
    // Act
    let response = refusal(json!([1, 2, 3]));

    // Assert
    assert_eq!(response.code, Some(ErrorCode::Malformed));
}

#[test]
fn a_reply_from_an_older_daemon_still_parses() {
    // Arrange -- only status and message, as before codes and data.
    let wire = b"{\"status\":\"ok\",\"message\":\"Daemon stopping\"}\n".to_vec();

    // Act
    let response: Response =
        read_message(&mut std::io::BufReader::new(std::io::Cursor::new(wire))).unwrap();

    // Assert
    assert_eq!(response, Response::ok_with_message("Daemon stopping"));
}

#[test]
fn an_error_code_from_a_newer_daemon_is_kept_as_other() {
    // Arrange
    let wire = json!({ "status": "error", "code": "quota-exceeded" });

    // Act
    let response: Response = serde_json::from_value(wire).unwrap();

    // Assert
    assert_eq!(response.code, Some(ErrorCode::Other));
}

#[test]
fn typed_data_round_trips() {
    // Arrange
    let status = DaemonStatus {
        version: "1.2.3".into(),
        protocol: 7,
        windows: 4,
    };

    // Act
    let response = Response::ok().with_data(&status);

    // Assert
    assert_eq!(response.data::<DaemonStatus>(), Some(status));
    assert_eq!(Response::ok().data::<DaemonStatus>(), None);
}

#[test]
fn a_command_that_could_not_be_carried_out_is_answered_failed() {
    // Arrange
    let error = crate::Error::platform("read the monitors", "no display");

    // Act
    let response = Response::failed(&error);

    // Assert
    assert_eq!(response.status, ResponseStatus::Error);
    assert_eq!(response.code, Some(ErrorCode::Failed));
    assert_eq!(
        response.message.as_deref(),
        Some("could not read the monitors: no display")
    );
}
