use std::sync::mpsc;

use mosaico_core::event::EventSource;
use mosaico_core::system::Interrupts;
use mosaico_core::tiler::Enumerator;

/// Watches window events in real time. Press Ctrl+C to stop, which
/// `interrupts` reports instead of ending the process. Each event that
/// names a window is shown with the title `desktop` reads for it.
pub fn execute(source: &dyn EventSource, interrupts: &dyn Interrupts, desktop: &dyn Enumerator) {
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
    if let Err(e) = interrupts.on_interrupt(stop_tx) {
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
                    let title = desktop.details(id).facts.title;
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
