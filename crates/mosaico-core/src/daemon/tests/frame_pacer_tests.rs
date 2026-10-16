//! The pacer's thread, against waits of the test's own.

use std::sync::atomic::AtomicUsize;
use std::sync::mpsc::Receiver;
use std::time::Instant;

use super::*;

#[path = "frame_pacer_stop_tests.rs"]
mod stop_tests;

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// A display that refreshes every 3 ms.
struct FastDisplay;

impl FrameWait for FastDisplay {
    fn display(&mut self) -> bool {
        thread::sleep(ms(3));
        true
    }

    fn until(&mut self, at: Instant) {
        thread::sleep(at.saturating_duration_since(Instant::now()));
    }

    fn now(&self) -> Instant {
        Instant::now()
    }

    fn refresh(&self) -> Option<Duration> {
        Some(ms(3))
    }
}

/// A display that does not come back for two seconds.
struct StuckDisplay;

impl FrameWait for StuckDisplay {
    fn display(&mut self) -> bool {
        thread::sleep(Duration::from_secs(2));
        true
    }

    fn until(&mut self, _: Instant) {}

    fn now(&self) -> Instant {
        Instant::now()
    }

    fn refresh(&self) -> Option<Duration> {
        None
    }
}

/// A pacer on the wait `make_wait` makes, and where its frames go.
fn pacer<W: FrameWait>(
    make_wait: impl FnOnce() -> W + Send + 'static,
) -> (FramePacer, Receiver<FrameToken>) {
    let (frames, received) = mpsc::channel();
    let send = move |token| frames.send(token).is_ok();
    let pacer = FramePacer::start(make_wait, send).expect("the thread starts");
    (pacer, received)
}

#[test]
fn no_frame_comes_until_frames_are_wanted() {
    // Arrange
    let (_pacer, frames) = pacer(|| FastDisplay);

    // Act
    let frame = frames.recv_timeout(ms(30));

    // Assert
    assert!(matches!(frame, Err(RecvTimeoutError::Timeout)));
}

#[test]
fn the_next_frame_comes_once_the_last_one_is_taken() {
    // Arrange -- asking twice is the same as asking once.
    let (pacer, frames) = pacer(|| FastDisplay);
    pacer.want_frames(true);
    pacer.want_frames(true);

    // Act
    let first = frames.recv_timeout(Duration::from_secs(1)).ok();
    let while_held = frames.recv_timeout(ms(30)).is_ok();
    let came = first.is_some();
    drop(first);
    let next = frames.recv_timeout(Duration::from_secs(1)).is_ok();

    // Assert
    assert!(came, "no frame came");
    assert!(!while_held, "a frame came before the last was taken");
    assert!(next, "no frame came once the last was taken");
}

#[test]
fn a_frame_that_cannot_be_sent_parks_the_pacer_until_frames_are_asked_for() {
    // Arrange -- the loop is gone, so no frame reaches it.
    let tries = Arc::new(AtomicUsize::new(0));
    let counted = Arc::clone(&tries);
    let send = move |_| {
        counted.fetch_add(1, Ordering::Relaxed);
        false
    };
    let pacer = FramePacer::start(|| FastDisplay, send).expect("the thread starts");
    let tried = |more_than: usize| soon(|| tries.load(Ordering::Relaxed) > more_than);

    // Act -- once it has tried, 60 ms is twenty frames.
    pacer.want_frames(true);
    let first = tried(0);
    thread::sleep(ms(60));
    let parked = tries.load(Ordering::Relaxed);
    pacer.want_frames(true);
    let again = tried(parked);

    // Assert
    assert!(first, "it never tried");
    assert!(parked < 3, "tried {parked} times");
    assert!(again, "it did not try again");
}

/// Whether `done` comes true within two seconds.
fn soon(done: impl Fn() -> bool) -> bool {
    let until = Instant::now() + Duration::from_secs(2);
    while !done() {
        if Instant::now() > until {
            return false;
        }
        thread::sleep(ms(1));
    }
    true
}
