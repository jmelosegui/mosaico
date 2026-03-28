use std::sync::mpsc::Sender;
use std::thread;

use mosaico_core::config::Keybinding;
use mosaico_core::{Action, WindowEvent, WindowResult};
use windows::Win32::Foundation::HWND;
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::Accessibility::{SetWinEventHook, UnhookWinEvent};
use windows::Win32::UI::WindowsAndMessaging::{
    PostThreadMessageW, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS, WM_QUIT,
};

use crate::hotkey::HotkeyManager;

#[path = "event_loop_event_sink.rs"]
mod event_loop_event_sink;
#[path = "event_loop_message_pump.rs"]
mod event_loop_message_pump;
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
/// Registers window event hooks and global hotkeys from the provided
/// keybinding configuration. Events and actions are sent through the
/// provided channels.
pub fn start(
    event_tx: Sender<WindowEvent>,
    action_tx: Sender<Action>,
    keybindings: Vec<Keybinding>,
    focus_follows_mouse: bool,
) -> WindowResult<EventLoopHandle> {
    let (ready_tx, ready_rx) = std::sync::mpsc::channel::<Result<(u32, usize), String>>();

    let handle = thread::spawn(move || {
        EVENT_SENDER.with(|cell| {
            *cell.borrow_mut() = Some(event_tx);
        });

        // SAFETY: GetCurrentThreadId returns the calling thread's ID.
        // It is always safe to call and never fails.
        let thread_id = unsafe { windows::Win32::System::Threading::GetCurrentThreadId() };

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
            let _ = ready_tx.send(Err("Failed to set WinEvent hook".to_string()));
            return;
        }

        // Register hotkeys on this thread's message queue.
        let mut hotkeys = HotkeyManager::new(action_tx);
        hotkeys.register_from_config(&keybindings);

        // Create a hidden window that receives display/work-area changes
        // and (optionally) focus-follows-mouse timer ticks.
        let event_sink =
            event_loop_event_sink::create_event_sink(focus_follows_mouse).unwrap_or_default();

        let _ = ready_tx.send(Ok((thread_id, event_sink.0 as usize)));

        // Run the message pump with hotkey dispatching.
        event_loop_message_pump::run_message_pump(&hotkeys);

        // Cleanup: destroy event sink, hotkeys unregistered in Drop.
        if !event_sink.is_invalid() {
            event_loop_event_sink::destroy_event_sink(event_sink, focus_follows_mouse);
        }
        drop(hotkeys);

        // SAFETY: UnhookWinEvent removes the hooks installed above.
        // Called during thread shutdown after the message pump exits.
        unsafe {
            let _ = UnhookWinEvent(hook_system);
            let _ = UnhookWinEvent(hook_object);
        }
    });

    let (thread_id, event_sink): (u32, usize) = ready_rx
        .recv()
        .map_err(|_| -> Box<dyn std::error::Error> {
            "event loop thread exited unexpectedly".into()
        })?
        .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;

    Ok(EventLoopHandle {
        thread_id,
        event_sink,
        handle,
    })
}

/// Handle for controlling the event loop from the daemon.
pub struct EventLoopHandle {
    thread_id: u32,
    event_sink: usize,
    handle: thread::JoinHandle<()>,
}

impl EventLoopHandle {
    /// Enables or disables focus-follows-mouse on the event loop thread.
    pub fn toggle_focus_follows_mouse(&self, enabled: bool) {
        if self.event_sink == 0 {
            return;
        }
        let hwnd = HWND(self.event_sink as *mut _);
        event_loop_event_sink::toggle_focus_follows_mouse(hwnd, enabled);
    }

    /// Signals the event loop to stop and waits for the thread to finish.
    pub fn stop(self) {
        // SAFETY: PostThreadMessageW sends WM_QUIT to the event loop
        // thread, causing GetMessageW to return false and the pump to
        // exit. The thread_id was obtained from GetCurrentThreadId on
        // the target thread.
        unsafe {
            let _ = PostThreadMessageW(self.thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
        }
        let _ = self.handle.join();
    }
}
