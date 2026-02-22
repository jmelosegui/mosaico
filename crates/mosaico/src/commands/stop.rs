use mosaico_core::ipc::ResponseStatus;

pub fn execute() {
    // Try graceful shutdown via IPC first.
    if mosaico_windows::ipc::is_daemon_running() {
        let command = mosaico_core::Command::Stop;
        match mosaico_windows::ipc::send_command(&command) {
            Ok(response) if response.status == ResponseStatus::Ok => {
                println!("Mosaico stopped. {}", response.message.unwrap_or_default());
                let _ = mosaico_core::pid::remove_pid_file();
                return;
            }
            Ok(response) => {
                eprintln!(
                    "Error: {}",
                    response.message.unwrap_or("unknown error".into())
                );
                return;
            }
            Err(e) => eprintln!("IPC failed: {e}"),
        }
    }

    // Fallback: the IPC pipe is gone but the process may still be
    // alive (e.g. the IPC thread crashed). Check the PID file.
    match mosaico_core::pid::read_pid_file() {
        Ok(Some(pid)) if mosaico_windows::process::is_process_alive(pid) => {
            if mosaico_windows::process::kill_process(pid) {
                let _ = mosaico_core::pid::remove_pid_file();
                println!("Mosaico stopped (killed PID {pid}).");
            } else {
                eprintln!("Failed to kill process {pid}.");
                std::process::exit(1);
            }
        }
        _ => {
            println!("Mosaico is not running.");
        }
    }
}
