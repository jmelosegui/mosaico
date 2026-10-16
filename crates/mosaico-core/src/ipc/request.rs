use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    COMMAND_NAMES, Command, DAEMON_VERSION, DaemonStatus, ErrorCode, PROTOCOL_VERSION, Response,
};

/// A command as it travels to the daemon, tagged with the protocol
/// version of the client that sent it.
///
/// The version sits next to the command's own fields, so a daemon that
/// predates versioning still reads the command and ignores the version.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Request {
    /// The client's [`PROTOCOL_VERSION`].
    pub version: u32,
    /// What the client asks for.
    #[serde(flatten)]
    pub command: Command,
}

impl Request {
    /// A request from this build.
    pub fn new(command: Command) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            command,
        }
    }
}

/// Reads a request the daemon received, or the reply that tells the
/// client why it cannot be carried out.
///
/// A request from another protocol version is refused with
/// [`ErrorCode::VersionMismatch`], except a stop, so a CLI from any
/// release can always stop the daemon and start its own.
pub fn decode_request(value: Value) -> Result<Command, Response> {
    if !value.is_object() {
        return Err(Response::error(
            ErrorCode::Malformed,
            "the request is not a JSON object",
        ));
    }
    let version = value.get("version").and_then(Value::as_u64);
    let command = Command::deserialize(&value);

    if version != Some(u64::from(PROTOCOL_VERSION)) {
        if let Ok(Command::Stop) = command {
            return Ok(Command::Stop);
        }
        return Err(version_mismatch(version));
    }

    command.map_err(|e| match value.get("command").and_then(Value::as_str) {
        // A name this build does not know is an unknown command. Anything
        // else that does not decode, such as a known command with bad
        // fields, is malformed.
        Some(name) if !COMMAND_NAMES.contains(&name) => Response::error(
            ErrorCode::UnknownCommand,
            format!("unknown command `{name}`"),
        ),
        _ => Response::error(ErrorCode::Malformed, format!("malformed request: {e}")),
    })
}

fn version_mismatch(client: Option<u64>) -> Response {
    let client = client.map_or_else(
        || "an unversioned".to_string(),
        |v| format!("a version {v}"),
    );
    Response::error(
        ErrorCode::VersionMismatch,
        format!(
            "the daemon (mosaico {DAEMON_VERSION}) speaks protocol version {PROTOCOL_VERSION}, \
             the client sent {client} request. Run `mosaico stop` and `mosaico start` \
             with the same mosaico as this command to restart it"
        ),
    )
    .with_data(&DaemonStatus {
        version: DAEMON_VERSION.to_string(),
        protocol: PROTOCOL_VERSION,
        windows: 0,
    })
}

#[cfg(test)]
#[path = "tests/request_tests.rs"]
mod tests;
