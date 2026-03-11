use mosaico_core::ipc::ResponseStatus;

/// Sends an Inspect command to the daemon and prints its workspace state.
pub fn execute() {
    if !mosaico_windows::ipc::is_daemon_running() {
        eprintln!("Mosaico is not running.");
        std::process::exit(1);
    }

    let command = mosaico_core::Command::Inspect;

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
