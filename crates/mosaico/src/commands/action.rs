use mosaico_core::Action;
use mosaico_core::ipc::ResponseStatus;

/// Sends an action to the running daemon via IPC.
pub fn execute(action: Action) {
    if !mosaico_windows::ipc::is_daemon_running() {
        eprintln!("Mosaico is not running.");
        std::process::exit(1);
    }

    let command = mosaico_core::Command::Action { action };

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
            eprintln!("Failed to send action: {e}");
            std::process::exit(1);
        }
    }
}
