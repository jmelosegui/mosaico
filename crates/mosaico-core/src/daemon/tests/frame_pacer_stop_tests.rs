//! Stopping the frames, and the pacer, whatever its thread is waiting
//! on.

use super::*;

#[test]
fn stopping_does_not_wait_for_a_display_that_does_not_come_back() {
    // Arrange -- the thread is waiting on the display as the pacer stops.
    let (pacer, _frames) = pacer(|| StuckDisplay);
    pacer.want_frames(true);
    thread::sleep(ms(20));

    // Act
    let started = Instant::now();
    drop(pacer);

    // Assert
    let took = started.elapsed();
    assert!(took < ms(500), "stopping took {took:?}");
}

#[test]
fn stopping_the_pacer_ends_its_thread() {
    // Arrange
    let (pacer, frames) = pacer(|| FastDisplay);
    pacer.want_frames(true);
    let _ = frames.recv_timeout(Duration::from_secs(1));

    // Act
    drop(pacer);

    // Assert -- the thread let go of where frames go as it ended.
    let ended = loop {
        if let Err(e) = frames.recv_timeout(Duration::from_secs(1)) {
            break e;
        }
    };
    assert_eq!(ended, RecvTimeoutError::Disconnected);
}

#[test]
fn stopping_a_pacer_that_was_never_asked_for_frames_ends_its_thread() {
    // Arrange -- the thread is parked, as it is for most of a session.
    let (pacer, frames) = pacer(|| FastDisplay);
    thread::sleep(ms(20));

    // Act
    drop(pacer);

    // Assert -- the thread let go of where frames go as it ended.
    let ended = frames.recv_timeout(Duration::from_secs(2));
    assert!(
        matches!(ended, Err(RecvTimeoutError::Disconnected)),
        "{ended:?}"
    );
}

#[test]
fn frames_stop_once_they_are_no_longer_wanted() {
    // Arrange
    let (pacer, frames) = pacer(|| FastDisplay);
    pacer.want_frames(true);
    let came = frames.recv_timeout(Duration::from_secs(1)).is_ok();

    // Act -- frames sent just before may still come, until none has
    // for 50 ms.
    pacer.want_frames(false);
    let late = (0..5).take_while(|_| frames.recv_timeout(ms(50)).is_ok());

    // Assert
    assert!(came, "no frame came");
    assert!(late.count() < 5, "frames kept coming");
}
