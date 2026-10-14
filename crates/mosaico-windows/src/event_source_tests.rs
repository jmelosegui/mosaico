//! The event thread, driven with stand-ins for the hooks and the hidden
//! window, so these tests install no hook and create no window.

use std::sync::mpsc::{self, RecvTimeoutError, TryRecvError};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use mosaico_core::{Error, WindowEvent, WindowId};

use super::{SetUp, send, set_up, watch};

/// How long a test waits for the event thread before it fails.
const WAIT: Duration = Duration::from_secs(5);

/// Writes its name to a shared log when it is dropped.
struct Dropped(&'static str, Arc<Mutex<Vec<&'static str>>>);

impl Drop for Dropped {
    fn drop(&mut self) {
        if let Ok(mut log) = self.1.lock() {
            log.push(self.0);
        }
    }
}

#[test]
fn what_the_callbacks_send_reaches_the_receiver() {
    // Arrange -- set_up runs on the event thread, like the callbacks.
    let (events, received) = mpsc::channel();

    // Act
    let watching = watch(events, || {
        send(WindowEvent::DisplayChanged);
        Ok(SetUp {
            owned: (),
            display_error: None,
        })
    })
    .expect("the event thread starts");

    // Assert
    assert_eq!(received.recv_timeout(WAIT), Ok(WindowEvent::DisplayChanged));
    watching.stop();
}

#[test]
fn once_the_watch_is_stopped_the_sender_is_gone() {
    // Arrange
    let (events, received) = mpsc::channel();
    let watching = watch(events, || {
        Ok(SetUp {
            owned: (),
            display_error: None,
        })
    })
    .expect("the event thread starts");

    // Act
    watching.stop();

    // Assert -- so a loop over the receiver, like the daemon's bridge, ends.
    assert_eq!(
        received.recv_timeout(WAIT),
        Err(RecvTimeoutError::Disconnected)
    );
}

#[test]
fn a_failed_set_up_fails_the_start_and_sends_nothing() {
    // Arrange
    let (events, received) = mpsc::channel();

    // Act
    let started = watch(events, || {
        Err::<SetUp<()>, _>(Error::Unsupported("watching windows"))
    });

    // Assert
    assert!(matches!(
        started,
        Err(Error::Unsupported("watching windows"))
    ));
    assert_eq!(received.try_recv(), Err(TryRecvError::Disconnected));
}

#[test]
fn the_hidden_window_goes_before_the_hooks() {
    // Arrange
    let log = Arc::new(Mutex::new(Vec::new()));
    let (hooks, window) = (Arc::clone(&log), Arc::clone(&log));
    let set_up = set_up(
        || Ok(Dropped("hooks", hooks)),
        || Ok(Dropped("window", window)),
    )
    .expect("both are set up");

    // Act
    drop(set_up.owned);

    // Assert
    assert_eq!(*log.lock().expect("the log"), vec!["window", "hooks"]);
}

#[test]
fn without_its_hidden_window_the_source_runs_and_says_why() {
    // Arrange -- review of #97: the hooks work, but the window that
    // reports display and work-area changes cannot be created.
    let (events, received) = mpsc::channel();
    let window = WindowId::from_raw(7);
    let no_window = || {
        let reason = "not enough memory";
        Err::<(), _>(Error::platform(
            "create the MosaicoEventSink window",
            reason,
        ))
    };

    // Act -- a window event as the hooks would report it.
    let watching = watch(events, move || {
        send(WindowEvent::Focused { window });
        set_up(|| Ok(()), no_window)
    })
    .expect("the source starts without the window");
    let error = watching.display_error().map(ToString::to_string);

    // Assert -- window events still come, and the caller can say why
    // display changes will not.
    assert_eq!(
        received.recv_timeout(WAIT),
        Ok(WindowEvent::Focused { window })
    );
    assert!(
        error
            .as_deref()
            .is_some_and(|e| e.starts_with("could not create the MosaicoEventSink window")),
        "the watch must say why there are no display changes, but got {error:?}"
    );
    watching.stop();
}
