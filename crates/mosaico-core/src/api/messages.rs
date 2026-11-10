//! What the daemon sends a bar on a session.

use serde::{Deserialize, Serialize};

use super::{Daemon, Focus, Monitor, Theme, Update};

/// One message from the daemon to a bar.
///
/// Each is a JSON object told apart by the key it carries: `welcome`,
/// `snapshot`, `update`, `ok`, `error` or `bye`. A bar ignores messages it
/// does not know, so later versions may add some.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum DaemonMessage {
    /// The answer to `hello`: the session is open.
    Welcome {
        /// What the daemon tells of itself and the session.
        welcome: Welcome,
    },
    /// Every followed topic in full, after `welcome`, and in answer to
    /// `subscribe` and `resync`, then with the request's `id`.
    Snapshot {
        /// The request answered, if any.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<u64>,
        /// The session's count of messages that carry state, from 1.
        seq: u64,
        /// The state's revision since the daemon started.
        rev: u64,
        /// The topics.
        snapshot: Snapshot,
    },
    /// What changed of the followed topics since the session's previous
    /// message that carried state.
    Update {
        /// The session's count of messages that carry state.
        seq: u64,
        /// The state's revision since the daemon started.
        rev: u64,
        /// What changed.
        update: Update,
    },
    /// A request was carried out.
    Ok {
        /// The request's `id`.
        id: u64,
        /// What came of it.
        ok: Ack,
    },
    /// A message could not be taken in. Without an `id` it concerns the
    /// session, or a message whose `id` could not be read.
    Error {
        /// The request refused, if known.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<u64>,
        /// Why.
        error: ApiError,
    },
    /// The daemon is ending the session.
    Bye {
        /// Why.
        bye: Bye,
    },
}

/// The opening of a session.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
#[serde(default)]
pub struct Welcome {
    /// The API version the session speaks.
    pub api: u32,
    /// mosaico's version.
    pub mosaico: String,
    /// Names the daemon's start: it changes when the daemon restarts.
    pub boot: String,
    /// The bar the user picked, as `[bar] provider` names it.
    pub provider: String,
    /// The topics the session follows.
    pub topics: Vec<String>,
}

/// Topics in full, each present only when followed.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
#[serde(default)]
pub struct Snapshot {
    /// The daemon.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub daemon: Option<Daemon>,
    /// The theme.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<Theme>,
    /// Every monitor, from the left.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monitors: Option<Vec<Monitor>>,
    /// Where the user works.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus: Option<Focus>,
}

/// What came of a request carried out.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
#[serde(default)]
pub struct Ack {
    /// For a request that may change the state, its revision once the
    /// request was applied. The session was sent a state at this
    /// revision or later before this reply.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rev: Option<u64>,
}

/// Why a message was refused.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub struct ApiError {
    /// What went wrong, for a program.
    pub code: ErrorCode,
    /// What went wrong, for a person.
    pub message: String,
    /// With `unsupported-api`, the versions the daemon speaks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api: Option<Vec<u32>>,
}

impl ApiError {
    /// The error `value` carries, read without the deserializer, so a
    /// client that keeps messages as JSON need not carry its code. A code
    /// this build does not know reads as [`ErrorCode::Other`].
    pub fn from_json(value: &serde_json::Value) -> Self {
        let code = value["code"]
            .as_str()
            .map_or(ErrorCode::Other, ErrorCode::named);
        let api = value["api"].as_array().map(|api| {
            let versions = api.iter().filter_map(serde_json::Value::as_u64);
            versions.filter_map(|v| u32::try_from(v).ok()).collect()
        });
        Self {
            code,
            message: value["message"].as_str().unwrap_or_default().to_string(),
            api,
        }
    }

    /// An error with `code` and `message`.
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            api: None,
        }
    }
}

/// What went wrong with a message, for a program. Later versions may add
/// codes; a bar reads one it does not know as [`ErrorCode::Other`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum ErrorCode {
    /// The daemon has as many sessions as it serves. Sent before closing.
    Busy,
    /// The daemon speaks none of the versions `hello` listed. Sent with
    /// the versions it speaks, before closing.
    UnsupportedApi,
    /// A message was not JSON, or not shaped as its kind is.
    Malformed,
    /// A message was larger than a message may be, sent before closing;
    /// or a reservation reached past a quarter of a monitor.
    TooLarge,
    /// A request of a kind the daemon does not know.
    UnknownRequest,
    /// An action a bar may not run: one that acts on the focused window.
    NotAllowed,
    /// The monitor named is not connected.
    NoMonitor,
    /// An action the daemon does not know.
    UnknownAction,
    /// The action was carried out and something it needed failed.
    Failed,
    /// The daemon is stopping and carries out nothing more.
    Stopping,
    /// The bar the session speaks for is not the one `[bar] provider`
    /// picks, so it may not reserve space.
    NotProvider,
    /// A code this build does not know.
    #[serde(other)]
    Other,
}

impl ErrorCode {
    /// Every code but `Other`, with its name on the wire.
    pub const NAMED: [(Self, &'static str); 11] = [
        (Self::Busy, "busy"),
        (Self::UnsupportedApi, "unsupported-api"),
        (Self::Malformed, "malformed"),
        (Self::TooLarge, "too-large"),
        (Self::UnknownRequest, "unknown-request"),
        (Self::NotAllowed, "not-allowed"),
        (Self::NoMonitor, "no-monitor"),
        (Self::UnknownAction, "unknown-action"),
        (Self::Failed, "failed"),
        (Self::Stopping, "stopping"),
        (Self::NotProvider, "not-provider"),
    ];

    /// The code `name` names on the wire, or `Other`.
    pub fn named(name: &str) -> Self {
        let found = Self::NAMED.iter().find(|(_, named)| *named == name);
        found.map_or(Self::Other, |(code, _)| *code)
    }
}

/// The end of a session.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
#[serde(default)]
pub struct Bye {
    /// Why it ends, such as `"stopping"`.
    pub reason: String,
}
