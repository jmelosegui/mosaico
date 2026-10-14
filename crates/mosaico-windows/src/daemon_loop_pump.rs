//! Keeps the windows of the daemon thread responsive.
//!
//! This thread owns the bar and border windows, so it has to pump its Win32
//! message queue. Windows classifies every window whose owning thread is not
//! pumping as not responding: the cursor turns into the busy spinner over the
//! bar, clicking it pops the "mosaico.exe is not responding" dialog, and the
//! desktop compositor paints an opaque ghost over what should be a
//! transparent layered bar. Draining the queue on every iteration, and on
//! every wait timeout, keeps those windows classified as alive.

use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, MSG, PM_REMOVE, PeekMessageW, TranslateMessage,
};

/// How long the loop waits for a daemon message before draining the
/// thread's Win32 message queue again.
pub(super) const PUMP_INTERVAL: std::time::Duration = std::time::Duration::from_millis(50);

/// Drains every message currently queued for this thread.
///
/// Non-blocking: it returns as soon as the queue is empty, so it is cheap to
/// call on every loop iteration. Each message is dispatched to the window
/// procedure that owns it, which on this thread means the bar and border
/// windows.
pub(super) fn pump_thread_messages() {
    let mut msg = MSG::default();

    // SAFETY: PeekMessageW removes one message from this thread's message
    // queue without blocking and returns false once the queue is empty. The
    // MSG struct is stack-allocated. TranslateMessage and DispatchMessageW
    // then route that message to the appropriate window procedure.
    unsafe {
        while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::System::Threading::GetCurrentThreadId;
    use windows::Win32::UI::WindowsAndMessaging::{PM_NOREMOVE, PostThreadMessageW, WM_USER};

    /// Looks at the queue without consuming anything.
    fn queue_is_empty() -> bool {
        let mut msg = MSG::default();

        // SAFETY: PM_NOREMOVE inspects this thread's queue without
        // removing a message. The MSG struct is stack allocated.
        unsafe { !PeekMessageW(&mut msg, None, 0, 0, PM_NOREMOVE).as_bool() }
    }

    #[test]
    fn pump_thread_messages_drains_the_queue() {
        // Arrange -- the first peek is also what creates the queue for
        // this thread, which PostThreadMessageW needs to exist.
        pump_thread_messages();

        // SAFETY: posting to the calling thread, which now has a queue.
        let thread_id = unsafe { GetCurrentThreadId() };
        for offset in 0..5 {
            // SAFETY: posting to a thread that has a message queue.
            unsafe {
                PostThreadMessageW(thread_id, WM_USER + offset, WPARAM(0), LPARAM(0))
                    .expect("failed to post a test message");
            }
        }
        assert!(!queue_is_empty(), "the test failed to queue anything");

        // Act
        pump_thread_messages();

        // Assert -- a queue left with anything in it is a thread Windows
        // will mark as not responding, which is the whole defect.
        assert!(
            queue_is_empty(),
            "pump_thread_messages left messages in the queue"
        );
    }
}
