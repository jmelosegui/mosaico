use std::sync::mpsc::Sender;
use std::thread;

use mosaico_core::{WindowEvent, WindowResult};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, MSG, PostThreadMessageW, TranslateMessage,
    WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS, WM_QUIT,
};

use crate::event;

/// Minimum event code we listen for (EVENT_SYSTEM_FOREGROUND = 0x0003).
const EVENT_MIN: u32 = 0x0003;

/// Maximum event code we listen for (EVENT_OBJECT_NAMECHANGE = 0x800C).
const EVENT_MAX: u32 = 0x800C;

// Thread-local sender for the WinEvent callback. The callback is a bare
// function pointer (no closures allowed), so we use thread-local storage
// to pass the channel sender into it.
thread_local! {
    static EVENT_SENDER: std::cell::RefCell<Option<Sender<WindowEvent>>> =
        const { std::cell::RefCell::new(None) };
}

/// Starts the Win32 event loop on a new thread.
///
/// Returns a `Sender` that can post `WM_QUIT` to stop the loop, and
/// the `JoinHandle` for the spawned thread.
///
/// Events are sent through the provided `tx` channel. The caller
/// (daemon) reads from the receiving end to process events.
pub fn start(tx: Sender<WindowEvent>) -> WindowResult<EventLoopHandle> {
    let (ready_tx, ready_rx) = std::sync::mpsc::channel::<Result<u32, String>>();

    let handle = thread::spawn(move || {
        // Store the sender in thread-local storage so the callback can use it.
        EVENT_SENDER.with(|cell| {
            *cell.borrow_mut() = Some(tx);
        });

        let thread_id = unsafe { windows::Win32::System::Threading::GetCurrentThreadId() };

        // SAFETY: SetWinEventHook registers our callback for system-wide
        // window events. WINEVENT_OUTOFCONTEXT means the callback runs in
        // our process (not injected into other processes).
        // WINEVENT_SKIPOWNPROCESS ignores events from our own windows.
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

        let _ = ready_tx.send(Ok(thread_id));

        // Run the Win32 message pump. GetMessageW blocks until a message
        // arrives. The pump is required because SetWinEventHook delivers
        // events via Windows messages — without pumping, the callback
        // never fires.
        run_message_pump();

        // SAFETY: UnhookWinEvent removes the hook we registered above.
        unsafe {
            let _ = UnhookWinEvent(hook);
        }
    });

    // Wait for the event loop thread to be ready.
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
        // SAFETY: PostThreadMessageW sends WM_QUIT to the event loop
        // thread, causing GetMessageW to return 0 and exit the pump.
        unsafe {
            let _ = PostThreadMessageW(self.thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
        }

        let _ = self.handle.join();
    }
}

/// The Win32 message pump. Blocks until WM_QUIT is received.
fn run_message_pump() {
    let mut msg = MSG::default();

    // SAFETY: GetMessageW blocks until a message is available.
    // Returns 0 when WM_QUIT is received, ending the loop.
    while unsafe { GetMessageW(&mut msg, None, 0, 0).as_bool() } {
        unsafe {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

/// The WinEvent callback. Called by Windows whenever a hooked event fires.
///
/// This is an `extern "system"` function pointer — no closures allowed.
/// We use thread-local storage to access the channel sender.
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
