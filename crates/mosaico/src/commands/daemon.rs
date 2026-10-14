use super::context::Exit;

/// Runs the daemon in this process until it is stopped.
pub fn execute() -> Exit {
    match mosaico_windows::daemon::run() {
        Ok(()) => Exit::Success,
        Err(e) => {
            eprintln!("Daemon error: {e}");
            Exit::Failure
        }
    }
}
