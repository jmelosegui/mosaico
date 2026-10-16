use std::sync::mpsc;
use std::time::Duration;

use mosaico_core::WindowControl;

use super::*;
use crate::window::test_threads::ThreadWindow;
use crate::window_control::Win32WindowControl;

#[test]
fn pre_place_flags_post_instead_of_sending() {
    // Arrange / Act / Assert -- the asynchronous flag is the whole
    // point of the pre placement move. Without it the call is
    // delivered synchronously and a window whose thread never pumps
    // blocks the daemon for good.
    assert_ne!(PRE_PLACE_FLAGS.0 & SWP_ASYNCWINDOWPOS.0, 0);
}

#[test]
fn pre_place_returns_even_when_the_owner_never_pumps() {
    // Arrange -- a window whose thread will never process messages.
    let Some(owned) = ThreadWindow::new() else {
        // No interactive desktop available, nothing to assert.
        return;
    };
    let window = owned.id();
    // Act -- run the move on its own thread so that a regression
    // fails this test instead of hanging it forever. A synchronous
    // SetWindowPos against this window never returns at all.
    let (done_tx, done_rx) = mpsc::channel::<()>();
    let mover = std::thread::spawn(move || {
        let _ = Win32WindowControl::default().pre_place(window, Rect::new(20, 20, 200, 200));
        let _ = done_tx.send(());
    });
    let returned = done_rx.recv_timeout(Duration::from_secs(2)).is_ok();

    // Cleanup before asserting so a failure cannot leak the window.
    drop(owned);
    if returned {
        let _ = mover.join();
    }

    // Assert
    assert!(
        returned,
        "pre_place did not return within 2s against a window whose thread never pumps, \
         which means the move was sent synchronously instead of posted"
    );
}
