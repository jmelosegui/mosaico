pub fn execute() {
    if let Err(e) = mosaico_windows::daemon::run() {
        eprintln!("Daemon error: {e}");
        std::process::exit(1);
    }
}
