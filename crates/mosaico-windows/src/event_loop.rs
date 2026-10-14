use std::sync::mpsc::Sender;

use mosaico_core::{Error, Result, WindowEvent};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Accessibility::{SetWinEventHook, UnhookWinEvent};
use windows::Win32::UI::WindowsAndMessaging::{WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS};

use crate::message_thread::{self, MessageThread};

#[path = "event_loop_event_sink.rs"]
mod event_loop_event_sink;
#[path = "event_loop_win_event.rs"]
mod event_loop_win_event;

/// System event range: FOREGROUND (0x0003) through MINIMIZEEND (0x0017).
///
/// Covers: EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_MOVESIZEEND,
/// EVENT_SYSTEM_MINIMIZESTART, EVENT_SYSTEM_MINIMIZEEND.
const SYSTEM_EVENT_MIN: u32 = 0x0003;
const SYSTEM_EVENT_MAX: u32 = 0x0017;

/// Object event range: CREATE (0x8000) through NAMECHANGE (0x800C).
///
/// Covers: EVENT_OBJECT_CREATE, EVENT_OBJECT_DESTROY, EVENT_OBJECT_SHOW,
/// EVENT_OBJECT_HIDE, EVENT_OBJECT_LOCATIONCHANGE, EVENT_OBJECT_NAMECHANGE.
const OBJECT_EVENT_MIN: u32 = 0x8000;
const OBJECT_EVENT_MAX: u32 = 0x800C;

// Thread-local sender for the WinEvent callback.
thread_local! {
    pub(super) static EVENT_SENDER: std::cell::RefCell<Option<Sender<WindowEvent>>> =
        const { std::cell::RefCell::new(None) };
}

/// Starts the Win32 event loop on a new thread.
///
/// Registers window event hooks and sends the events through the
/// provided channel.
///
/// Fails when the hooks cannot be installed. The loop runs without the
/// window that reports display and work area changes if that cannot be
/// created; [`EventLoopHandle::sink_error`] says why.
pub fn start(event_tx: Sender<WindowEvent>) -> Result<EventLoopHandle> {
    let (thread, sink_error) = MessageThread::spawn("window events", move |ready| {
        EVENT_SENDER.with(|cell| {
            *cell.borrow_mut() = Some(event_tx);
        });

        let flags = WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS;

        // Use two targeted hooks instead of one broad range (0x0003–0x800C)
        // to avoid receiving thousands of irrelevant events in the gap
        // (0x0018–0x7FFF) that translate() would just discard. Under heavy
        // system load this noise can starve the message pump.
        // SAFETY: SetWinEventHook installs out-of-process event hooks
        // for system and object events. The callback function pointer is
        // valid for the lifetime of the thread. Hooks are unhooked on exit.
        let hook_system = unsafe {
            SetWinEventHook(
                SYSTEM_EVENT_MIN,
                SYSTEM_EVENT_MAX,
                None,
                Some(event_loop_win_event::win_event_proc),
                0,
                0,
                flags,
            )
        };
        // SAFETY: Second SetWinEventHook for object events (same safety as above).
        let hook_object = unsafe {
            SetWinEventHook(
                OBJECT_EVENT_MIN,
                OBJECT_EVENT_MAX,
                None,
                Some(event_loop_win_event::win_event_proc),
                0,
                0,
                flags,
            )
        };

        if hook_system.is_invalid() || hook_object.is_invalid() {
            for hook in [hook_system, hook_object] {
                if !hook.is_invalid() {
                    // SAFETY: UnhookWinEvent removes the hook that was
                    // installed above, so a failed start leaves none.
                    unsafe {
                        let _ = UnhookWinEvent(hook);
                    }
                }
            }
            let reason = "SetWinEventHook failed";
            ready.report(Err(Error::platform("set the window event hooks", reason)));
            return;
        }

        // Create a hidden window that receives display/work-area changes.
        // Without it the loop still runs, it just misses those, so the
        // caller is told why instead of the start failing.
        let (event_sink, sink_error) = match event_loop_event_sink::create_event_sink() {
            Ok(sink) => (sink, None),
            Err(e) => (HWND::default(), Some(e)),
        };

        ready.report(Ok(sink_error));

        message_thread::pump();

        // Cleanup: destroy the event sink, then the hooks.
        if !event_sink.is_invalid() {
            event_loop_event_sink::destroy_event_sink(event_sink);
        }

        // SAFETY: UnhookWinEvent removes the hooks installed above.
        // Called during thread shutdown after the message pump exits.
        unsafe {
            let _ = UnhookWinEvent(hook_system);
            let _ = UnhookWinEvent(hook_object);
        }
        EVENT_SENDER.with(|cell| cell.borrow_mut().take());
    })?;

    Ok(EventLoopHandle { thread, sink_error })
}

/// Handle for stopping the event loop.
pub struct EventLoopHandle {
    thread: MessageThread,
    sink_error: Option<Error>,
}

impl EventLoopHandle {
    /// Why the hidden window that reports display and work area changes
    /// could not be created. The loop runs without it, so those changes
    /// never arrive.
    pub fn sink_error(&self) -> Option<&Error> {
        self.sink_error.as_ref()
    }

    /// Signals the event loop to stop and waits for the thread to finish.
    pub fn stop(self) {
        self.thread.stop();
    }
}

#[cfg(test)]
#[path = "event_loop_tests.rs"]
mod tests;
