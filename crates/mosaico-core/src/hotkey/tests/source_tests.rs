use std::sync::{Arc, Mutex};

use crate::hotkey::{HotkeyControl, HotkeyWatch};
use crate::{Error, Result};

/// What a source was asked to do, oldest first.
type Calls = Arc<Mutex<Vec<&'static str>>>;

/// A platform control that records what it is asked, and can fail to
/// reach its source.
struct Recording {
    calls: Calls,
    reachable: bool,
}

impl Recording {
    fn record(&self, call: &'static str) -> Result<()> {
        self.calls.lock().expect("the calls").push(call);
        if self.reachable {
            Ok(())
        } else {
            Err(Error::platform("reach the hotkeys", "the thread is gone"))
        }
    }
}

impl HotkeyControl for Recording {
    fn pause(&self) -> Result<()> {
        self.record("pause")
    }

    fn unpause(&self) -> Result<()> {
        self.record("unpause")
    }

    fn stop(self: Box<Self>) {
        let _ = self.record("stop");
    }
}

fn watch(reachable: bool) -> (HotkeyWatch, Calls) {
    let calls = Calls::default();
    let control = Recording {
        calls: Arc::clone(&calls),
        reachable,
    };
    (HotkeyWatch::new(control), calls)
}

fn calls(calls: &Calls) -> Vec<&'static str> {
    calls.lock().expect("the calls").clone()
}

#[test]
fn a_watch_leaves_the_hotkeys_running_until_it_is_stopped() {
    // Act
    let (watch, recorded) = watch(true);

    // Assert
    assert!(calls(&recorded).is_empty());
    drop(watch);
}

#[test]
fn pausing_and_unpausing_reach_the_source_in_order() {
    // Arrange
    let (watch, recorded) = watch(true);

    // Act
    let paused = watch.pause();
    let unpaused = watch.unpause();

    // Assert
    assert!(paused.is_ok() && unpaused.is_ok());
    assert_eq!(calls(&recorded), ["pause", "unpause"]);
}

#[test]
fn a_pause_that_cannot_reach_the_source_is_an_error() {
    // Arrange
    let (watch, _) = watch(false);

    // Act
    let result = watch.pause();

    // Assert
    assert!(matches!(result, Err(Error::Platform { .. })));
}

#[test]
fn stopping_a_watch_stops_the_source_once() {
    // Arrange
    let (watch, recorded) = watch(true);

    // Act
    watch.stop();

    // Assert
    assert_eq!(calls(&recorded), ["stop"]);
}

#[test]
fn dropping_a_watch_stops_the_source() {
    // Arrange
    let (watch, recorded) = watch(true);

    // Act
    drop(watch);

    // Assert
    assert_eq!(calls(&recorded), ["stop"]);
}
