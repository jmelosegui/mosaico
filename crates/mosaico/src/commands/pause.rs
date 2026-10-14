use super::request;

/// Sends a PauseHotkeys command to the running daemon.
pub fn pause() {
    send(mosaico_core::Command::PauseHotkeys);
}

/// Sends an UnpauseHotkeys command to the running daemon.
pub fn unpause() {
    send(mosaico_core::Command::UnpauseHotkeys);
}

fn send(command: mosaico_core::Command) {
    request::print_message(request::send_or_exit(&command));
}
