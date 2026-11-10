//! What a bar sends the daemon on a session.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::StableMonitorId;

use super::{ApiError, ErrorCode, Release, Reserve};

/// The versions of the API this build speaks, oldest first.
pub const SUPPORTED_APIS: [u32; 1] = [super::API_VERSION];

/// One message from a bar to the daemon.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum ClientMessage {
    /// The first message of a session.
    Hello {
        /// What the bar tells of itself.
        hello: Hello,
    },
    /// A request, answered with its `id`.
    Request {
        /// Picked by the bar, carried back on the reply.
        id: u64,
        /// What is asked.
        #[serde(flatten)]
        request: Request,
    },
}

/// The opening of a session.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
#[serde(default)]
pub struct Hello {
    /// The API versions the bar speaks.
    pub api: Vec<u32>,
    /// The bar the session speaks for, such as `"yasb"`, or `None` for any
    /// other.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bar: Option<String>,
    /// The program speaking, for the log and `doctor`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client: Option<String>,
    /// The topics to follow, or `None` for all of them.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topics: Option<Vec<String>>,
    /// The process that draws the bar, when it is not the one connected,
    /// as when a bar speaks through `mosaico subscribe`. Its windows are
    /// not tiled while the session reserves space.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process: Option<u32>,
}

/// What a bar asks of the daemon.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum Request {
    /// Runs an action.
    Run(Run),
    /// Follows these topics from now on, answered with a snapshot of them.
    Subscribe(Subscribe),
    /// Sends a snapshot of the topics followed, after a gap in `seq`.
    Resync {},
    /// Answers at once, to show the session is alive.
    Ping {},
    /// Keeps tiles clear of the bar's space on a monitor, in place of what
    /// the session reserved there before.
    Reserve(Reserve),
    /// Ends the session's reservations.
    Release(Release),
}

/// An action to run, as a click on a monitor's bar runs it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub struct Run {
    /// The action, as `bar.toml`'s click bindings name it, such as
    /// `"goto-workspace-3"`.
    pub action: String,
    /// The monitor it acts on, by id, or `None` for the focused one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, schemars(with = "Option<String>"))]
    pub monitor: Option<StableMonitorId>,
}

/// The topics to follow.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
#[serde(default)]
pub struct Subscribe {
    /// Their names. Names the daemon does not know are left out.
    pub topics: Vec<String>,
}

/// The request kinds this build knows, as their keys on the wire.
const KINDS: [&str; 6] = ["run", "subscribe", "resync", "ping", "reserve", "release"];

/// Why a message from a bar was refused, and the request's `id` when it
/// could be read.
#[derive(Debug, Clone, PartialEq)]
pub struct Refusal {
    /// The request refused, if its `id` could be read.
    pub id: Option<u64>,
    /// Why.
    pub error: ApiError,
}

impl ClientMessage {
    /// Reads a message from `value`, one line of the session.
    ///
    /// A request of a kind this build does not know is refused with
    /// `unknown-request` and its `id`, so the bar learns which one failed
    /// and the session goes on.
    pub fn parse(value: Value) -> Result<Self, Refusal> {
        let Value::Object(mut map) = value else {
            return Err(refusal(None, ErrorCode::Malformed, "not a JSON object"));
        };
        if let Some(hello) = map.remove("hello") {
            return serde_json::from_value(hello)
                .map(|hello| Self::Hello { hello })
                .map_err(|e| refusal(None, ErrorCode::Malformed, format!("bad hello: {e}")));
        }
        let id = match map.remove("id") {
            Some(id) => id.as_u64(),
            None => None,
        };
        let Some(id) = id else {
            return Err(refusal(None, ErrorCode::Malformed, "no whole number `id`"));
        };
        let request = parse_request(map).map_err(|error| Refusal {
            id: Some(id),
            error,
        })?;
        Ok(Self::Request { id, request })
    }

    /// The message as the line a client sends: what serializing it gives,
    /// built from its parts, as the derived form of an untagged message
    /// with a flattened field costs a client much more code.
    pub fn to_json(&self) -> Value {
        let built = match self {
            Self::Hello { hello } => {
                serde_json::to_value(hello).map(|hello| json!({"hello": hello}))
            }
            Self::Request { id, request } => serde_json::to_value(request).map(|mut line| {
                line["id"] = json!(id);
                line
            }),
        };
        // Neither part has a map with keys that are not strings, the one
        // thing that fails to serialize into a value.
        built.unwrap_or_default()
    }
}

/// The request in `map`, whose `id` was taken out.
fn parse_request(mut map: Map<String, Value>) -> Result<Request, ApiError> {
    let kind = KINDS.into_iter().find(|kind| map.contains_key(*kind));
    let Some(kind) = kind else {
        return Err(match map.keys().next() {
            Some(kind) => ApiError::new(
                ErrorCode::UnknownRequest,
                format!("unknown request `{kind}`"),
            ),
            None => ApiError::new(ErrorCode::Malformed, "no request beside `id`"),
        });
    };
    // Each kind is read on its own: the derived reading of the enum costs
    // the daemon much more code.
    let body = map.remove(kind).unwrap_or_default();
    match kind {
        "run" => read(kind, body).map(Request::Run),
        "subscribe" => read(kind, body).map(Request::Subscribe),
        "reserve" => read(kind, body).map(Request::Reserve),
        "release" => read(kind, body).map(Request::Release),
        _ if !body.is_object() => Err(malformed(kind, "expected an object")),
        "resync" => Ok(Request::Resync {}),
        _ => Ok(Request::Ping {}),
    }
}

/// The body of a request of kind `kind`.
fn read<T: DeserializeOwned>(kind: &str, body: Value) -> Result<T, ApiError> {
    serde_json::from_value(body).map_err(|e| malformed(kind, e))
}

fn malformed(kind: &str, why: impl std::fmt::Display) -> ApiError {
    ApiError::new(ErrorCode::Malformed, format!("bad {kind}: {why}"))
}

/// The highest version in `offered` this build speaks too.
pub fn negotiate(offered: &[u32]) -> Option<u32> {
    (offered.iter().copied())
        .filter(|api| SUPPORTED_APIS.contains(api))
        .max()
}

fn refusal(id: Option<u64>, code: ErrorCode, message: impl Into<String>) -> Refusal {
    Refusal {
        id,
        error: ApiError::new(code, message),
    }
}
