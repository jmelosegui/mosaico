use mosaico_core::{Action, Command};

use super::context::{Context, Exit};
use super::request;

/// Sends an action to the running daemon.
pub fn execute(cx: &mut Context, action: Action) -> Exit {
    request::run(cx, &Command::Action { action })
}
