use std::sync::mpsc::Sender;
use std::thread;

use mosaico_core::config::Keybinding;
use mosaico_core::{Action, WindowEvent, WindowResult};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, MSG, PostThreadMessageW, TranslateMessage,
    WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS, WM_HOTKEY, WM_QUIT,
};

use crate::event;
use crate::hotkey::HotkeyManager;

/// Minimum event code we listen for (EVENT_SYSTEM_FOREGROUND = 0x0003).
const EVENT_MIN: u32 = 0x0003;

/// Maximum event code we listen for (EVENT_OBJECT_NAMECHANGE = 0x800C).
const EVENT_MAX: u32 = 0x800C;

// Thread-local sender for the WinEvent callback.
thread_local! {
    static EVENT_SENDER: std::cell::RefCell<Option<Sender<WindowEvent>>> =
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
                Some(win_event_proc),
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

        let _ = ready_tx.send(Ok(thread_id));

        // Run the message pump with hotkey dispatching.
        run_message_pump(&hotkeys);

        // Cleanup: hotkeys are unregistered in HotkeyManager::drop.
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

/// The Win32 message pump. Dispatches hotkey messages and blocks
/// until WM_QUIT is received.
fn run_message_pump(hotkeys: &HotkeyManager) {
    let mut msg = MSG::default();

    while unsafe { GetMessageW(&mut msg, None, 0, 0).as_bool() } {
        if msg.message == WM_HOTKEY {
            hotkeys.dispatch(msg.wParam.0 as i32);
            continue;
        }

        unsafe {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

/// The WinEvent callback.
unsafe extern "system" fn win_event_proc(
    _hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    id_object: i32,
    _id_child: i32,
    _event_thread: u32,
    _event_time: u32,
) {
    if let Some(window_event) = event::translate(event, hwnd, id_object) {
        EVENT_SENDER.with(|cell| {
            if let Some(sender) = cell.borrow().as_ref() {
                let _ = sender.send(window_event);
            }
        });
    }
}
