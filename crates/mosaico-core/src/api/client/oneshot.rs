//! `query` and `run`: one session, one answer.

use std::io::Write;
use std::time::Duration;

use serde_json::Value;

use crate::StableMonitorId;
use crate::api::{ApiError, ClientMessage, ErrorCode, Request, Run};
use crate::ipc::framing::write_message;

use super::link::{Event, hello, not_connected, read_daemon, refused, session_error};
use super::queue::{Received, Receiver, queue};
use super::{ApiClient, FieldPath, JsonState, MonitorRef, Outcome, field_text};

/// How long a one-off command waits for each message of the daemon.
const ANSWER_WAIT: Duration = Duration::from_secs(10);

/// What `query` prints.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Query {
    /// The topics, or `None` for all of them, or for the one `field` is in.
    pub topics: Option<Vec<String>>,
    /// The only monitor to show.
    pub monitor: Option<MonitorRef>,
    /// The one value to print, as text, instead of the state as JSON.
    pub field: Option<FieldPath>,
}

/// Prints the state once, as `client`, and returns.
pub fn query(client: &dyn ApiClient, name: &str, query: &Query, out: &mut dyn Write) -> Outcome {
    let field_topic = (query.field.as_ref()).map(|field| vec![field.topic().to_string()]);
    let topics = query.topics.clone().or(field_topic);
    let result = Link::open(client, &hello(name, None, topics)).and_then(|link| link.snapshot());
    let state = match result {
        Ok(state) => state,
        Err(outcome) => return outcome,
    };
    let monitor = query.monitor.as_ref();
    let line = match &query.field {
        Some(field) => field_text(field.lookup(&state.view(None, monitor))),
        None => state.view(query.topics.as_deref(), monitor).to_string(),
    };
    let _ = writeln!(out, "{line}");
    Outcome::Done
}

/// Runs `action` on `monitor`, or on the focused monitor, as `client`, and
/// returns once the daemon answers.
pub fn run(
    client: &dyn ApiClient,
    name: &str,
    action: &str,
    monitor: Option<&MonitorRef>,
) -> Outcome {
    match run_in(client, name, action, monitor) {
        Ok(outcome) | Err(outcome) => outcome,
    }
}

fn run_in(
    client: &dyn ApiClient,
    name: &str,
    action: &str,
    monitor: Option<&MonitorRef>,
) -> Result<Outcome, Outcome> {
    // An index is turned into an id from the monitors; an id is passed on.
    let index = matches!(monitor, Some(MonitorRef::Index(_)));
    let topics = if index {
        vec!["monitors".to_string()]
    } else {
        Vec::new()
    };
    let mut link = Link::open(client, &hello(name, None, Some(topics)))?;
    let state = link.snapshot()?;
    let monitor = match monitor {
        None => None,
        Some(MonitorRef::Id(id)) => Some(id.clone()),
        Some(named) => Some(state.monitor_id(named).ok_or_else(|| {
            let message = "no monitor is at that index";
            Outcome::Refused(ApiError::new(ErrorCode::NoMonitor, message))
        })?),
    };
    let request = ClientMessage::Request {
        id: 1,
        request: Request::Run(Run {
            action: action.to_string(),
            monitor: monitor.map(StableMonitorId::new),
        }),
    };
    write_message(&mut link.writer, &request.to_json()).map_err(|_| Outcome::Lost)?;
    let reply = link.until(|message| message["id"] == 1)?;
    match reply.get("error") {
        None => Ok(Outcome::Done),
        Some(error) => Err(Outcome::Refused(ApiError::from_json(error))),
    }
}

/// One session, read in order.
struct Link {
    events: Receiver<Event>,
    writer: Box<dyn Write + Send>,
}

impl Link {
    fn open(client: &dyn ApiClient, hello: &ClientMessage) -> Result<Self, Outcome> {
        let connection = client.connect().map_err(not_connected)?;
        let (sender, events) = queue();
        read_daemon(connection.reader, 0, &sender);
        let mut writer = connection.writer;
        write_message(&mut writer, &hello.to_json()).map_err(|_| Outcome::Lost)?;
        Ok(Self { events, writer })
    }

    /// The state of the first snapshot.
    fn snapshot(&self) -> Result<JsonState, Outcome> {
        let message = self.until(|message| message.get("snapshot").is_some())?;
        let mut state = JsonState::default();
        state.reset(&message["snapshot"]);
        Ok(state)
    }

    /// The next message `wanted` picks. Fails when the session ends, an
    /// error ends it, or nothing comes in time.
    fn until(&self, wanted: impl Fn(&Value) -> bool) -> Result<Value, Outcome> {
        loop {
            match self.events.recv_timeout(ANSWER_WAIT) {
                Received::Item(Event::Daemon(_, message)) => {
                    if let Some(error) = session_error(&message) {
                        return Err(refused(error));
                    }
                    if wanted(&message) {
                        return Ok(message);
                    }
                }
                _ => return Err(Outcome::Lost),
            }
        }
    }
}
