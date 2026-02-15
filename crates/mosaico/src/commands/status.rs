pub fn execute() {
    if mosaico_windows::ipc::is_daemon_running() {
        println!("Mosaico is running.");
        return;
    }

    // Pipe isn't responding — check if a stale PID file was left behind
    // by a daemon that was killed without a clean shutdown.
    if let Ok(Some(pid)) = mosaico_core::pid::read_pid_file() {
        if mosaico_windows::process::is_process_alive(pid) {
            println!("Mosaico process exists (PID: {pid}) but is not responding.");
        } else {
            let _ = mosaico_core::pid::remove_pid_file();
            println!("Mosaico is not running (cleaned up stale PID file).");
        }
    } else {
        println!("Mosaico is not running.");
    }
}
