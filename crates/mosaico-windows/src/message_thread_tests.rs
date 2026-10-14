use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use mosaico_core::Error;
use windows::Win32::UI::WindowsAndMessaging::{GetMessageW, MSG, WM_USER};

use super::{MessageThread, Ready, pump};

/// How long a test waits for the thread before it fails.
const WAIT: Duration = Duration::from_secs(5);

/// A body that reports success, pumps until it is stopped, and then
/// records that it took down what it owned.
fn pumping(torn_down: &Arc<AtomicBool>) -> impl FnOnce(Ready<()>) + Send + 'static {
    let torn_down = Arc::clone(torn_down);
    move |ready| {
        ready.report(Ok(()));
        pump();
        torn_down.store(true, Ordering::SeqCst);
    }
}

#[test]
fn a_message_posted_as_soon_as_it_started_is_received() {
    // Arrange -- a body that passes on what reaches its queue.
    let (seen_tx, seen) = mpsc::channel();
    let (thread, ()) = MessageThread::spawn("test", move |ready| {
        ready.report(Ok(()));
        let mut msg = MSG::default();
        // SAFETY: GetMessageW writes into the stack allocated MSG and
        // returns false for WM_QUIT.
        while unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
            let _ = seen_tx.send((msg.message, msg.wParam.0));
        }
    })
    .expect("the thread starts");

    // Act -- posting right away relies on the queue existing already.
    thread
        .post(WM_USER + 7, 42)
        .expect("the thread has a queue");

    // Assert
    assert_eq!(seen.recv_timeout(WAIT), Ok((WM_USER + 7, 42)));
    thread.stop();
}

#[test]
fn what_the_setup_reports_reaches_the_caller() {
    // Act -- such as a part of the setup that failed without stopping it.
    let (thread, reported) = MessageThread::spawn("test", |ready| {
        ready.report(Ok("no hidden window"));
        pump();
    })
    .expect("the thread starts");

    // Assert
    assert_eq!(reported, "no hidden window");
    thread.stop();
}

#[test]
fn a_failed_setup_is_returned_by_spawn() {
    // Act
    let started = MessageThread::spawn("test", |ready: Ready<()>| {
        ready.report(Err(Error::Unsupported("watching windows")));
    });

    // Assert
    assert!(matches!(
        started,
        Err(Error::Unsupported("watching windows"))
    ));
}

#[test]
fn a_body_that_ends_without_reporting_fails_to_start() {
    // Act
    let started = MessageThread::spawn("test", |_ready: Ready<()>| {});

    // Assert
    let error = started.err().expect("the thread never got ready");
    assert_eq!(
        error.to_string(),
        "could not start a message thread: the test thread ended before it was ready"
    );
}

#[test]
fn stopping_ends_the_pump_and_runs_what_follows_it() {
    // Arrange
    let torn_down = Arc::new(AtomicBool::new(false));
    let (thread, ()) =
        MessageThread::spawn("test", pumping(&torn_down)).expect("the thread starts");

    // Act
    thread.stop();

    // Assert
    assert!(torn_down.load(Ordering::SeqCst));
}

#[test]
fn stopping_while_the_queue_is_flooded_still_ends_the_thread() {
    // Arrange -- a burst of messages queued ahead of the request to quit.
    let torn_down = Arc::new(AtomicBool::new(false));
    let (thread, ()) =
        MessageThread::spawn("test", pumping(&torn_down)).expect("the thread starts");
    for i in 0..5_000 {
        thread.post(WM_USER + 1, i).expect("room in the queue");
    }

    // Act
    thread.stop();

    // Assert
    assert!(torn_down.load(Ordering::SeqCst));
}

#[test]
fn stopping_a_thread_whose_queue_is_full_waits_until_it_has_ended() {
    // Arrange -- a thread too busy to pump while its queue fills up to
    // the limit Windows sets, so WM_QUIT cannot be posted at first.
    let torn_down = Arc::new(AtomicBool::new(false));
    let (release_tx, release) = mpsc::channel::<()>();
    let flag = Arc::clone(&torn_down);
    let (thread, ()) = MessageThread::spawn("test", move |ready| {
        ready.report(Ok(()));
        let _ = release.recv();
        pump();
        flag.store(true, Ordering::SeqCst);
    })
    .expect("the thread starts");
    let mut queued = 0;
    while thread.post(WM_USER + 1, 0).is_ok() {
        queued += 1;
        assert!(queued < 1_000_000, "the queue never filled up");
    }
    // The thread goes back to its pump a while after stop is called.
    let busy = thread::spawn(move || {
        thread::sleep(Duration::from_millis(200));
        let _ = release_tx.send(());
    });

    // Act
    thread.stop();

    // Assert -- stop waited for the thread instead of returning while it
    // still ran and held what it owns.
    assert!(torn_down.load(Ordering::SeqCst));
    busy.join().expect("the busy spell ends");
}

#[test]
fn dropping_it_stops_the_thread() {
    // Arrange
    let torn_down = Arc::new(AtomicBool::new(false));
    let (thread, ()) =
        MessageThread::spawn("test", pumping(&torn_down)).expect("the thread starts");

    // Act
    drop(thread);

    // Assert
    assert!(torn_down.load(Ordering::SeqCst));
}

#[test]
fn stopping_a_thread_that_already_ended_returns() {
    // Arrange -- a body that is done as soon as it is ready.
    let (ended_tx, ended) = mpsc::channel();
    let (thread, ()) = MessageThread::spawn("test", move |ready| {
        ready.report(Ok(()));
        let _ = ended_tx.send(());
    })
    .expect("the thread starts");
    ended.recv_timeout(WAIT).expect("the body ended");

    // Act / Assert -- returns instead of waiting for a pump that is gone.
    thread.stop();
}
