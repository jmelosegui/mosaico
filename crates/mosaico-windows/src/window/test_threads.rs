//! Windows of the tests' own, which are never shown, on a thread of
//! their own that handles its messages when told to, or never, and the
//! threads that move them.

use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use mosaico_core::{Rect, WindowId};
use windows::Win32::UI::HiDpi::{
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetThreadDpiAwarenessContext,
};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, MSG, PM_REMOVE, PeekMessageW, TranslateMessage,
};

use super::Window;

#[path = "test_thread_window.rs"]
mod thread_window;

pub(crate) use thread_window::ThreadWindow;

/// Runs `act` on a thread of its own, and returns what it returned, or
/// `None` when it did not return within a second. A move sent instead of
/// posted to a window whose thread never handles its messages never
/// returns, which fails the test rather than hanging it.
pub(crate) fn within_a_second<T: Send + 'static>(
    act: impl FnOnce() -> T + Send + 'static,
) -> Option<T> {
    let (done, returned) = mpsc::channel();
    thread::spawn(move || {
        let _ = done.send(act());
    });
    returned.recv_timeout(Duration::from_secs(1)).ok()
}

/// Waits up to two seconds for the window's visible bounds to be other
/// than `from`, without moving it, and says whether they got there.
pub(crate) fn moved_from(window: WindowId, from: Option<Rect>) -> bool {
    within_two_seconds(|| Window::from_id(window).rect().ok() != from)
}

/// Waits up to two seconds for `holds` to hold, and says whether it did.
pub(crate) fn within_two_seconds(holds: impl Fn() -> bool) -> bool {
    let until = Instant::now() + Duration::from_secs(2);
    while !holds() {
        if Instant::now() > until {
            return false;
        }
        thread::sleep(Duration::from_millis(5));
    }
    true
}

/// Makes the calling thread per-monitor DPI aware, as the daemon is, so
/// it reads and moves windows in the display's own pixels.
pub(crate) fn per_monitor_dpi_aware() {
    // SAFETY: changes only the DPI awareness of the calling thread.
    unsafe {
        let _ = SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }
}

/// Handles every message queued for the calling thread.
fn pump() {
    let mut msg = MSG::default();
    // SAFETY: PeekMessageW takes a message off this thread's queue
    // without blocking, and TranslateMessage and DispatchMessageW hand it
    // to the procedure of its window.
    unsafe {
        while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
