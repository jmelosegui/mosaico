use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, MSG, TranslateMessage, WM_HOTKEY,
};

use crate::hotkey::HotkeyManager;

/// The Win32 message pump. Dispatches hotkey messages and blocks
/// until WM_QUIT is received.
pub(crate) fn run_message_pump(hotkeys: &HotkeyManager) {
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
