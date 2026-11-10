//! What the client commands share: the daemon read on a thread of its
//! own, the opening `hello`, and how a command ends.

use std::io::{BufRead, BufReader, Read};
use std::thread;

use serde_json::{Value, json};

use crate::api::{ApiError, ErrorCode, Hello, SUPPORTED_APIS};
use crate::ipc::IpcError;
use crate::ipc::framing::read_message;

use super::OpenInput;
use super::queue::Sender;

/// How a client command ended.
#[derive(Debug)]
pub enum Outcome {
    /// It did what was asked, or its session ended as it should: the
    /// daemon said `bye`, the input or the output closed, the process
    /// that started it exited, or no request came in time.
    Done,
    /// The daemon refused what was asked.
    Refused(ApiError),
    /// No daemon runs.
    NotRunning,
    /// The daemon went away without `bye`, or did not answer in time.
    Lost,
    /// The daemon speaks none of the API versions this build speaks; the
    /// ones it does.
    Mismatch(Vec<u32>),
    /// The connection could not be made.
    Failed(IpcError),
}

/// What a client command waits on.
pub(super) enum Event {
    /// A message of the session `generation` counts.
    Daemon(u64, Value),
    /// The end of that session.
    Ended(u64),
    /// A line of the input.
    Input(String),
    /// The input ended.
    InputEnded,
    /// The process that started this one exited.
    ParentExited,
}

/// The `hello` that opens a session for `client`, speaking for the bar
/// that runs as `process`, if any, as the line sent.
pub(super) fn hello(
    client: &str,
    bar: Option<&str>,
    topics: Option<Vec<String>>,
    process: Option<u32>,
) -> Value {
    let hello = Hello {
        api: SUPPORTED_APIS.to_vec(),
        bar: bar.map(str::to_string),
        client: Some(client.to_string()),
        topics,
        process,
    };
    json!({ "hello": hello })
}

/// Reads the daemon's messages on `reader` on a thread of its own, and
/// sends each, then the end, as events of session `generation`. A line
/// that is not JSON is skipped.
pub(super) fn read_daemon(reader: Box<dyn Read + Send>, generation: u64, events: &Sender<Event>) {
    let sender = events.clone();
    let spawned = (thread::Builder::new())
        .name("bar-api-client".into())
        .spawn(move || {
            let mut reader = BufReader::new(reader);
            loop {
                match read_message::<Value>(&mut reader) {
                    Ok(message) => {
                        if sender.send(Event::Daemon(generation, message)).is_err() {
                            return;
                        }
                    }
                    Err(IpcError::Malformed(_)) => {}
                    Err(_) => {
                        let _ = sender.send(Event::Ended(generation));
                        return;
                    }
                }
            }
        });
    if spawned.is_err() {
        let _ = events.send(Event::Ended(generation));
    }
}

/// The error a message without an `id` carries, which concerns the
/// session as a whole.
pub(super) fn session_error(message: &Value) -> Option<ApiError> {
    if message.get("id").is_some() {
        return None;
    }
    message.get("error").map(ApiError::from_json)
}

/// How a command ends on `error`, which ended its session.
pub(super) fn refused(error: ApiError) -> Outcome {
    match error.code {
        ErrorCode::UnsupportedApi => Outcome::Mismatch(error.api.unwrap_or_default()),
        _ => Outcome::Refused(error),
    }
}

/// How a command ends when it could not connect.
pub(super) fn not_connected(error: IpcError) -> Outcome {
    match error {
        IpcError::NotRunning => Outcome::NotRunning,
        error => Outcome::Failed(error),
    }
}

/// Runs `body` on a thread named `name`.
pub(super) fn spawn(name: &str, body: impl FnOnce() + Send + 'static) {
    let _ = thread::Builder::new().name(name.into()).spawn(body);
}

/// Sends each line of the input that `open` opens, then its end.
pub(super) fn read_input(open: OpenInput, sender: &Sender<Event>) {
    for line in open().lines() {
        let Ok(line) = line else { break };
        if !line.trim().is_empty() && sender.send(Event::Input(line)).is_err() {
            return;
        }
    }
    let _ = sender.send(Event::InputEnded);
}

/// The answer to request `id`, which the daemon was lost before answering.
pub(super) fn disconnected(id: u64) -> String {
    let message = "the daemon went away before answering";
    json!({"id": id, "error": {"code": "disconnected", "message": message}}).to_string()
}
