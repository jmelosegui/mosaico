use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::id::wid;
use crate::{Error, Result, WindowEvent, WindowId};

use super::{FocusFollowsMouse, MousePointer, POLL_INTERVAL, REST_POLLS};

/// How long a test waits for the polling thread before it fails.
const WAIT: Duration = Duration::from_secs(5);

/// A pointer resting on the same window for ever, counting its polls.
struct Resting {
    window: WindowId,
    polls: Arc<AtomicUsize>,
}

impl Resting {
    fn on(raw: u64) -> Self {
        Self {
            window: wid(raw),
            polls: Arc::default(),
        }
    }
}

impl MousePointer for Resting {
    fn window_under(&self) -> Result<Option<WindowId>> {
        self.polls.fetch_add(1, Ordering::SeqCst);
        Ok(Some(self.window))
    }
}

/// A pointer that gives scripted answers, then finds no window.
struct Scripted(Mutex<VecDeque<Result<Option<WindowId>>>>);

impl MousePointer for Scripted {
    fn window_under(&self) -> Result<Option<WindowId>> {
        let mut answers = self.0.lock().expect("the script lock");
        answers.pop_front().unwrap_or(Ok(None))
    }
}

/// A pointer that rests on window 7 for two polls, then finds no window
/// until the test brings it back to window 7 for a single poll.
#[derive(Default)]
struct Interrupted {
    polls: AtomicUsize,
    back: AtomicBool,
    returned: AtomicBool,
}

impl MousePointer for Arc<Interrupted> {
    fn window_under(&self) -> Result<Option<WindowId>> {
        if self.polls.fetch_add(1, Ordering::SeqCst) < 2 {
            return Ok(Some(wid(7)));
        }
        if self.back.load(Ordering::SeqCst) && !self.returned.swap(true, Ordering::SeqCst) {
            return Ok(Some(wid(7)));
        }
        Ok(None)
    }
}

/// Focus-follows-mouse polling as fast as it can, so tests do not wait.
fn fast(pointer: impl MousePointer + 'static) -> (FocusFollowsMouse, mpsc::Receiver<WindowEvent>) {
    let (events, received) = mpsc::channel();
    (
        FocusFollowsMouse::polling_every(Duration::ZERO, pointer, events),
        received,
    )
}

fn hover(raw: u64) -> WindowEvent {
    WindowEvent::MouseHover { window: wid(raw) }
}

#[test]
fn the_pointer_must_rest_three_polls_of_100_ms() {
    // Assert -- the feel of focus-follows-mouse depends on these.
    assert_eq!(POLL_INTERVAL, Duration::from_millis(100));
    assert_eq!(REST_POLLS, 3);
}

#[test]
fn it_starts_off_without_a_thread() {
    // Arrange
    let pointer = Resting::on(7);
    let polls = pointer.polls.clone();

    // Act
    let (ffm, _received) = fast(pointer);

    // Assert
    assert!(!ffm.is_enabled());
    drop(ffm);
    assert_eq!(polls.load(Ordering::SeqCst), 0);
}

#[test]
fn turned_on_it_reports_the_window_the_pointer_rests_on() {
    // Arrange
    let (mut ffm, received) = fast(Resting::on(7));

    // Act
    ffm.set_enabled(true).expect("the polling thread starts");

    // Assert
    assert_eq!(received.recv_timeout(WAIT), Ok(hover(7)));
}

#[test]
fn turning_it_off_stops_the_polling_thread() {
    // Arrange -- on, and seen polling.
    let (mut ffm, received) = fast(Resting::on(7));
    ffm.set_enabled(true).expect("the polling thread starts");
    received.recv_timeout(WAIT).expect("a first report");

    // Act
    ffm.set_enabled(false).expect("turning off cannot fail");

    // Assert -- the thread ended and let go of the pointer.
    assert!(!ffm.is_enabled());
    assert_eq!(Arc::strong_count(&ffm.pointer), 1);
}

#[test]
fn turning_it_on_again_starts_over() {
    // Arrange -- reported once, then turned off, which forgets the rest.
    let (mut ffm, received) = fast(Resting::on(7));
    ffm.set_enabled(true).expect("the polling thread starts");
    received.recv_timeout(WAIT).expect("a first report");
    ffm.set_enabled(false).expect("turning off cannot fail");

    // Act
    ffm.set_enabled(true)
        .expect("the polling thread starts again");

    // Assert -- the same window, reported again by a new thread.
    assert_eq!(received.recv_timeout(WAIT), Ok(hover(7)));
}

#[test]
fn turning_it_on_while_it_is_on_keeps_a_rest_in_progress() {
    // Arrange -- two polls find window 7, then the pointer is away.
    let pointer = Arc::new(Interrupted::default());
    let (mut ffm, received) = fast(Arc::clone(&pointer));
    ffm.set_enabled(true).expect("the polling thread starts");
    let deadline = Instant::now() + WAIT;
    while pointer.polls.load(Ordering::SeqCst) < 3 {
        assert!(Instant::now() < deadline, "the pointer was not polled");
        thread::yield_now();
    }

    // Act -- a config reload with focus-follows-mouse still on, then the
    // pointer is back on window 7 for one poll.
    ffm.set_enabled(true).expect("already on");
    pointer.back.store(true, Ordering::SeqCst);

    // Assert -- that poll is the third of the rest begun before the
    // reload, which a restarted poller would have forgotten.
    assert_eq!(received.recv_timeout(WAIT), Ok(hover(7)));
    assert_eq!(Arc::strong_count(&ffm.pointer), 2, "one polling thread");
}

#[test]
fn dropping_it_stops_the_thread_and_lets_go_of_the_sender() {
    // Arrange
    let (mut ffm, received) = fast(Resting::on(7));
    ffm.set_enabled(true).expect("the polling thread starts");

    // Act
    drop(ffm);

    // Assert -- every sender is gone, so a loop over the receiver ends.
    loop {
        match received.recv_timeout(WAIT) {
            Ok(event) => assert_eq!(event, hover(7)),
            Err(RecvTimeoutError::Disconnected) => break,
            Err(RecvTimeoutError::Timeout) => panic!("a sender outlived the drop"),
        }
    }
}

#[test]
fn a_pointer_that_cannot_be_read_is_skipped() {
    // Arrange -- one failed read amid a rest on window 7. Were it to
    // break the rest, the script would end before three polls in a row.
    let answers = VecDeque::from([
        Ok(Some(wid(7))),
        Ok(Some(wid(7))),
        Err(Error::Unsupported("reading the pointer")),
        Ok(Some(wid(7))),
    ]);
    let (mut ffm, received) = fast(Scripted(Mutex::new(answers)));

    // Act
    ffm.set_enabled(true).expect("the polling thread starts");

    // Assert
    assert_eq!(received.recv_timeout(WAIT), Ok(hover(7)));
}
