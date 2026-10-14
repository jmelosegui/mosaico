use crate::commands::request;

/// Sends an Inspect command to the daemon and prints its workspace state.
pub fn execute() {
    let response = request::send_or_exit(&mosaico_core::Command::Inspect);
    request::print_message(response);
}
