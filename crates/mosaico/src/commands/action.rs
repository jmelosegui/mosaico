use mosaico_core::Action;

use super::request;

/// Sends an action to the running daemon via IPC.
pub fn execute(action: Action) {
    let response = request::send_or_exit(&mosaico_core::Command::Action { action });
    request::print_message(response);
}
