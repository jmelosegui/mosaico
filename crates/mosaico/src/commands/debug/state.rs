use mosaico_core::Command;

use crate::commands::context::{Context, Exit};
use crate::commands::request;

/// Prints the daemon's internal workspace state.
pub fn execute(cx: &mut Context) -> Exit {
    request::run(cx, &Command::Inspect)
}
