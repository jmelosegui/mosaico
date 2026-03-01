use std::sync::mpsc::Sender;
use std::thread;

use mosaico_core::config::Keybinding;
use mosaico_core::{Action, WindowEvent, WindowResult};
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

/// Minimum event code we listen for (EVENT_SYSTEM_FOREGROUND = 0x0003).
const EVENT_MIN: u32 = 0x0003;

/// Maximum event code we listen for (EVENT_OBJECT_NAMECHANGE = 0x800C).
const EVENT_MAX: u32 = 0x800C;

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
) -> WindowResult<EventLoopHandle> {
    let (ready_tx, ready_rx) = std::sync::mpsc::channel::<Result<u32, String>>();

    let handle = thread::spawn(move || {
        EVENT_SENDER.with(|cell| {
            *cell.borrow_mut() = Some(event_tx);
        });

        let thread_id = unsafe { windows::Win32::System::Threading::GetCurrentThreadId() };

        // SAFETY: SetWinEventHook registers our callback for system-wide
        // window events. WINEVENT_OUTOFCONTEXT means the callback runs in
        // our process. WINEVENT_SKIPOWNPROCESS ignores our own windows.
        let hook = unsafe {
            SetWinEventHook(
                EVENT_MIN,
                EVENT_MAX,
                None,
                Some(event_loop_win_event::win_event_proc),
                0,
                0,
                WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
            )
        };

        if hook.is_invalid() {
            let _ = ready_tx.send(Err("Failed to set WinEvent hook".to_string()));
            return;
        }

        // Register hotkeys on this thread's message queue.
        let mut hotkeys = HotkeyManager::new(action_tx);
        hotkeys.register_from_config(&keybindings);

        // Create a message-only window that receives WM_DISPLAYCHANGE.
        let event_sink = event_loop_event_sink::create_event_sink();

        let _ = ready_tx.send(Ok(thread_id));

        // Run the message pump with hotkey dispatching.
        event_loop_message_pump::run_message_pump(&hotkeys);

        // Cleanup: destroy event sink, hotkeys unregistered in Drop.
        if let Some(hwnd) = event_sink {
            event_loop_event_sink::destroy_event_sink(hwnd);
        }
        drop(hotkeys);

        unsafe {
            let _ = UnhookWinEvent(hook);
        }
    });

    let thread_id: u32 = ready_rx
        .recv()
        .map_err(|_| -> Box<dyn std::error::Error> {
            "event loop thread exited unexpectedly".into()
        })?
        .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;

    Ok(EventLoopHandle { thread_id, handle })
}

/// Handle for controlling the event loop from the daemon.
pub struct EventLoopHandle {
    thread_id: u32,
    handle: thread::JoinHandle<()>,
}

impl EventLoopHandle {
    /// Signals the event loop to stop and waits for the thread to finish.
    pub fn stop(self) {
        unsafe {
            let _ = PostThreadMessageW(self.thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
        }
        let _ = self.handle.join();
    }
}
