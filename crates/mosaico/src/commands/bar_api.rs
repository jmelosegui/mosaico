//! What `query`, `subscribe` and `run` share: they speak the bar API
//! rather than the CLI protocol, so they keep working with a daemon of
//! another release that speaks its version.

use std::io::Write;
use std::sync::Arc;

use mosaico_core::api::SUPPORTED_APIS;
use mosaico_core::api::client::{ApiClient, OpenInput, Outcome};
use mosaico_core::ipc::IpcError;
use mosaico_core::system::ParentWatch;

use super::context::Exit;
use super::request::NOT_RESPONDING;

/// What a bar API command may use.
pub struct BarIo<'a> {
    /// The daemon's bar API.
    pub api: &'a dyn ApiClient,
    /// Standard input, for `subscribe --requests`.
    pub input: &'a dyn Fn() -> OpenInput,
    /// The process that started this one, made only for `subscribe`.
    pub parent: &'a dyn Fn() -> Option<Arc<dyn ParentWatch>>,
    /// Standard output.
    pub out: &'a mut dyn Write,
    /// Standard error.
    pub err: &'a mut dyn Write,
}

/// Tells why a command ended unless it did what was asked, and the exit.
pub fn finish(outcome: Outcome, err: &mut dyn Write) -> Exit {
    let complaint = match outcome {
        Outcome::Done => return Exit::Success,
        Outcome::Refused(error) => format!("Error: {}", error.message),
        Outcome::NotRunning => "Mosaico is not running.".to_string(),
        Outcome::Lost => "Mosaico went away without saying why.".to_string(),
        Outcome::Mismatch(theirs) => {
            let _ = writeln!(
                err,
                "Mosaico speaks bar API version {}, and this command speaks {}. \
                 Use the mosaico command of the running release.",
                versions(&theirs),
                versions(&SUPPORTED_APIS),
            );
            return Exit::Mismatch;
        }
        Outcome::Failed(IpcError::TimedOut) => NOT_RESPONDING.to_string(),
        Outcome::Failed(error) => format!("Failed to reach the daemon: {error}"),
    };
    let _ = writeln!(err, "{complaint}");
    Exit::Failure
}

fn versions(api: &[u32]) -> String {
    let listed: Vec<String> = api.iter().map(u32::to_string).collect();
    if listed.is_empty() {
        "no version".to_string()
    } else {
        listed.join(", ")
    }
}

#[cfg(test)]
#[path = "tests/bar_api_tests.rs"]
mod tests;
