use std::io::{self, BufRead, Cursor};
use std::sync::Arc;

use mosaico_core::api::client::{ApiClient, ClientConnection, OpenInput, Outcome};
use mosaico_core::api::{ApiError, ErrorCode};
use mosaico_core::ipc::IpcError;
use mosaico_core::system::ParentWatch;

use super::{BarIo, finish};
use crate::commands::context::Exit;
use crate::commands::query::{self, QueryArgs};

fn finished(outcome: Outcome) -> (Exit, String) {
    let mut err = Vec::new();
    let exit = finish(outcome, &mut err);
    (exit, String::from_utf8(err).unwrap())
}

/// A daemon that sends `script` and ends.
struct Scripted(&'static str);

impl ApiClient for Scripted {
    fn connect(&self) -> Result<ClientConnection, IpcError> {
        Ok(ClientConnection {
            reader: Box::new(Cursor::new(self.0.as_bytes())),
            writer: Box::new(io::sink()),
        })
    }
}

#[test]
fn a_version_mismatch_exits_with_3_naming_the_versions_each_side_speaks() {
    // Arrange, Act
    let (exit, err) = finished(Outcome::Mismatch(vec![2, 3]));

    // Assert
    assert_eq!((exit, exit.code()), (Exit::Mismatch, 3));
    assert_eq!(
        err,
        "Mosaico speaks bar API version 2, 3, and this command speaks 1. \
         Use the mosaico command of the running release.\n"
    );
}

#[test]
fn every_other_failure_exits_with_1_and_says_why() {
    // Arrange
    let refused = ApiError::new(ErrorCode::NotAllowed, "a bar may not close windows");

    // Act
    let ended = [
        finished(Outcome::Done),
        finished(Outcome::Refused(refused)),
        finished(Outcome::NotRunning),
        finished(Outcome::Lost),
    ];

    // Assert
    let expected = [
        (Exit::Success, ""),
        (Exit::Failure, "Error: a bar may not close windows\n"),
        (Exit::Failure, "Mosaico is not running.\n"),
        (Exit::Failure, "Mosaico went away without saying why.\n"),
    ];
    let ended: Vec<(Exit, &str)> = ended.iter().map(|(e, s)| (*e, s.as_str())).collect();
    assert_eq!(ended, expected);
}

#[test]
fn query_prints_the_field_asked_for() {
    // Arrange
    let daemon = Scripted(concat!(
        r#"{"welcome":{"api":1}}"#,
        "\n",
        r#"{"seq":1,"rev":4,"snapshot":{"focus":{"window":{"title":"a\nb"}}}}"#,
        "\n",
    ));
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let no_input = || -> OpenInput { Box::new(|| Box::new(io::empty()) as Box<dyn BufRead>) };
    let no_parent = || -> Option<Arc<dyn ParentWatch>> { None };
    let mut io = BarIo {
        api: &daemon,
        input: &no_input,
        parent: &no_parent,
        out: &mut out,
        err: &mut err,
    };
    let args = QueryArgs {
        topics: Vec::new(),
        monitor: None,
        field: Some("focus.window.title".parse().unwrap()),
    };

    // Act
    let exit = query::execute(&mut io, args);

    // Assert
    assert_eq!(exit, Exit::Success);
    assert_eq!(String::from_utf8(out).unwrap(), "a\\nb\n");
    assert!(err.is_empty());
}
