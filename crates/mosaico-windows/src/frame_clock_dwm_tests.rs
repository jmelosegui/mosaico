//! The core pacer on DWM, briefly, as the daemon will start it. The
//! pacer's own logic is tested in core, against waits of the tests' own.

use std::sync::mpsc::{self, RecvTimeoutError};

use mosaico_core::daemon::{FrameClock, FramePacer};

use super::*;

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

#[test]
fn frames_follow_the_display_while_they_are_wanted() {
    // Arrange
    let (frames, received) = mpsc::channel();
    let send = move |token| frames.send(token).is_ok();
    let pacer = FramePacer::start(DisplayWait::new, send).expect("the thread starts");

    // Act -- each frame is taken as it comes. Frames sent just before
    // they are turned off may still come, until none has for 50 ms.
    pacer.want_frames(true);
    let started = Instant::now();
    let came = (0..3).all(|_| received.recv_timeout(ms(500)).is_ok());
    let took = started.elapsed();
    pacer.want_frames(false);
    let late = (0..5).take_while(|_| received.recv_timeout(ms(50)).is_ok());
    let late = late.count();
    let after = received.recv_timeout(ms(100));

    // Assert -- at least a millisecond apart whatever the display, so
    // never a busy loop.
    assert!(came, "frames did not come");
    assert!(took >= ms(2), "three frames came in {took:?}");
    assert!(late < 5, "frames kept coming");
    assert!(matches!(after, Err(RecvTimeoutError::Timeout)));
}

#[test]
fn the_refresh_dwm_reports_is_that_of_a_display() {
    // Arrange
    let wait = DisplayWait::new();

    // Act
    let refresh = wait.refresh();

    // Assert -- from 1000 Hz to 10 Hz, or none where nothing is composed.
    if let Some(refresh) = refresh {
        assert!(refresh >= ms(1) && refresh <= ms(100), "{refresh:?}");
    }
}
