use std::sync::mpsc::Sender;
use std::thread;

use mosaico_core::config::Keybinding;
use mosaico_core::{Action, WindowEvent, WindowResult};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW, MSG,
    PostThreadMessageW, RegisterClassW, TranslateMessage, WM_DISPLAYCHANGE, WM_HOTKEY, WM_QUIT,
    WNDCLASSW, WS_EX_TOOLWINDOW, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS,
};
use windows::core::w;

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

        // Create a message-only window that receives WM_DISPLAYCHANGE.
        let event_sink = create_event_sink();

        let _ = ready_tx.send(Ok(thread_id));

        // Run the message pump with hotkey dispatching.
        run_message_pump(&hotkeys);

        // Cleanup: destroy event sink, hotkeys unregistered in Drop.
        if let Some(hwnd) = event_sink {
            unsafe {
                let _ = DestroyWindow(hwnd);
            }
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

/// Creates a hidden window that receives `WM_DISPLAYCHANGE`.
///
/// Must NOT be a message-only window (`HWND_MESSAGE` parent) because those
/// do not receive broadcast messages. Instead we create a regular hidden
/// window with `WS_EX_TOOLWINDOW` to keep it out of the taskbar.
fn create_event_sink() -> Option<HWND> {
    unsafe {
        let class_name = w!("MosaicoEventSink");
        let wc = WNDCLASSW {
            lpfnWndProc: Some(event_sink_proc),
            lpszClassName: class_name.into(),
            ..Default::default()
        };

        if RegisterClassW(&wc) == 0 {
            eprintln!("Failed to register MosaicoEventSink class");
            return None;
        }

        // WS_EX_TOOLWINDOW: no taskbar entry.
        // No WS_VISIBLE: window stays hidden.
        // No HWND_MESSAGE parent: window receives broadcast messages.
        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW,
            class_name,
            w!("MosaicoEventSink"),
            Default::default(),
            0,
            0,
            0,
            0,
            None,
            None,
            None,
            None,
        );

        match hwnd {
            Ok(h) if !h.is_invalid() => Some(h),
            _ => {
                eprintln!("Failed to create MosaicoEventSink window");
                None
            }
        }
    }
}

/// WNDPROC for the event sink window.
///
/// Catches `WM_DISPLAYCHANGE` and sends a `DisplayChanged` event through
/// the thread-local sender. All other messages are passed to `DefWindowProcW`.
unsafe extern "system" fn event_sink_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_DISPLAYCHANGE {
        EVENT_SENDER.with(|cell| {
            if let Some(sender) = cell.borrow().as_ref() {
                let _ = sender.send(WindowEvent::DisplayChanged);
            }
        });
    }
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
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
