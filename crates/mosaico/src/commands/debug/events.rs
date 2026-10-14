use std::sync::mpsc;

use mosaico_core::event::EventSource;
use mosaico_windows::Window as WinWindow;

/// Watches window events in real time. Press Ctrl+C to stop.
pub fn execute(source: &dyn EventSource) {
    println!("Watching window events (press Ctrl+C to stop)...\n");

    let (tx, rx) = mpsc::channel();

    let watch = match source.start(tx) {
        Ok(watch) => watch,
        Err(e) => {
            eprintln!("Failed to start event loop: {e}");
            return;
        }
    };
    if let Some(e) = watch.display_error() {
        eprintln!("warning: display and work area changes will not be shown: {e}");
    }

    // Set up Ctrl+C handler to stop the event loop cleanly.
    let (stop_tx, stop_rx) = mpsc::channel();
    if let Err(e) = mosaico_windows::ctrl_c::set_handler(stop_tx) {
        eprintln!("warning: {e}");
    }

    loop {
        // Check for Ctrl+C
        if stop_rx.try_recv().is_ok() {
            break;
        }

        // Drain available events
        match rx.recv_timeout(std::time::Duration::from_millis(100)) {
            Ok(event) => match event.window() {
                Some(id) => {
                    let title = WinWindow::from_id(id).title().unwrap_or_default();
                    println!("{event} \"{title}\"");
                }
                None => println!("{event}"),
            },
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    watch.stop();
}
