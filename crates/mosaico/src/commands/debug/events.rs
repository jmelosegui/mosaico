use std::sync::mpsc;

/// Watches window events in real time. Press Ctrl+C to stop.
pub fn execute() {
    println!("Watching window events (press Ctrl+C to stop)...\n");

    let (tx, rx) = mpsc::channel();
    // Action channel is unused in debug mode — no hotkeys needed.
    let (action_tx, _action_rx) = mpsc::channel();

    let event_loop = match mosaico_windows::event_loop::start(tx, action_tx, Vec::new()) {
        Ok(handle) => handle,
        Err(e) => {
            eprintln!("Failed to start event loop: {e}");
            return;
        }
    };

    // Set up Ctrl+C handler to stop the event loop cleanly.
    let (stop_tx, stop_rx) = mpsc::channel();
    ctrlc::set_handler(move || {
        let _ = stop_tx.send(());
    })
    .expect("failed to set Ctrl+C handler");

    loop {
        // Check for Ctrl+C
        if stop_rx.try_recv().is_ok() {
            break;
        }

        // Drain available events
        match rx.recv_timeout(std::time::Duration::from_millis(100)) {
            Ok(event) => println!("{event:?}"),
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    event_loop.stop();
}
