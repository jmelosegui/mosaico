use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, MSG, PM_REMOVE, PeekMessageW, TranslateMessage, WM_HOTKEY,
};

use crate::hotkey::HotkeyManager;

/// The Win32 message pump. Prioritises hotkey messages so that
/// keyboard shortcuts remain responsive even when the event queue
/// is flooded (e.g. during a virus scan or heavy WPF event storm).
pub(crate) fn run_message_pump(hotkeys: &HotkeyManager) {
    let mut msg = MSG::default();

    loop {
        // Drain all pending WM_HOTKEY messages first so shortcuts
        // are never starved by a backlog of WinEvent callbacks.
        // SAFETY: PeekMessageW drains WM_HOTKEY messages from the thread's
        // message queue without blocking. The MSG struct is stack-allocated.
        while unsafe { PeekMessageW(&mut msg, None, WM_HOTKEY, WM_HOTKEY, PM_REMOVE).as_bool() } {
            hotkeys.dispatch(msg.wParam.0 as i32);
        }

        // Block until the next message arrives, then process it.
        // SAFETY: GetMessageW blocks until a message arrives. Returns
        // false on WM_QUIT, signaling the pump to exit.
        if !unsafe { GetMessageW(&mut msg, None, 0, 0).as_bool() } {
            break; // WM_QUIT
        }

        if msg.message == WM_HOTKEY {
            hotkeys.dispatch(msg.wParam.0 as i32);
            continue;
        }

        // SAFETY: TranslateMessage and DispatchMessageW process a single
        // message from the thread's queue, dispatching to the appropriate
        // window procedure (WinEvent hook or event sink).
        unsafe {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
