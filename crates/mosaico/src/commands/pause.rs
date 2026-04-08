use mosaico_core::ipc::ResponseStatus;

/// Sends a PauseHotkeys command to the running daemon.
pub fn pause() {
    send(mosaico_core::Command::PauseHotkeys);
}

/// Sends an UnpauseHotkeys command to the running daemon.
pub fn unpause() {
    send(mosaico_core::Command::UnpauseHotkeys);
}

fn send(command: mosaico_core::Command) {
    if !mosaico_windows::ipc::is_daemon_running() {
        eprintln!("Mosaico is not running.");
        std::process::exit(1);
    }

    match mosaico_windows::ipc::send_command(&command) {
        Ok(response) => {
            if response.status == ResponseStatus::Ok {
                if let Some(msg) = response.message {
                    println!("{msg}");
                }
            } else {
                eprintln!(
                    "Error: {}",
                    response.message.unwrap_or("unknown error".into()),
                );
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Failed to send command: {e}");
            std::process::exit(1);
        }
    }
}
