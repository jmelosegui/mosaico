use mosaico_core::ipc::ResponseStatus;

pub fn execute() {
    if !mosaico_windows::ipc::is_daemon_running() {
        println!("Mosaico is not running.");
        return;
    }

    let command = mosaico_core::Command::Stop;

    match mosaico_windows::ipc::send_command(&command) {
        Ok(response) => {
            if response.status == ResponseStatus::Ok {
                println!("Mosaico stopped. {}", response.message.unwrap_or_default());
            } else {
                eprintln!(
                    "Error: {}",
                    response.message.unwrap_or("unknown error".into())
                );
            }
        }
        Err(e) => eprintln!("Failed to send stop command: {e}"),
    }
}
