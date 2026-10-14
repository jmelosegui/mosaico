use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassW, WNDCLASSW, WS_EX_TOOLWINDOW,
};
use windows::core::w;

use mosaico_core::{Error, Result};

use super::EVENT_SENDER;
use crate::error::OrPlatform;

/// Creates a hidden window that receives `WM_DISPLAYCHANGE` and
/// `WM_SETTINGCHANGE`.
///
/// Must NOT be a message-only window (`HWND_MESSAGE` parent) because those
/// do not receive broadcast messages. Instead we create a regular hidden
/// window with `WS_EX_TOOLWINDOW` to keep it out of the taskbar.
pub(crate) fn create_event_sink() -> Result<HWND> {
    // SAFETY: Registers a window class and creates a hidden popup window
    // for receiving broadcast messages (WM_DISPLAYCHANGE, WM_SETTINGCHANGE).
    // All Win32 calls use valid parameters; the HWND is checked before use.
    unsafe {
        let class_name = w!("MosaicoEventSink");
        let wc = WNDCLASSW {
            lpfnWndProc: Some(event_sink_proc),
            lpszClassName: class_name,
            ..Default::default()
        };

        if RegisterClassW(&wc) == 0 {
            return Err(Error::platform(
                "register the MosaicoEventSink window class",
                windows::core::Error::from_win32(),
            ));
        }

        // WS_EX_TOOLWINDOW: no taskbar entry.
        // No WS_VISIBLE: window stays hidden.
        // No HWND_MESSAGE parent: window receives broadcast messages.
        // An Ok handle is never null.
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
        )
        .or_platform("create the MosaicoEventSink window")?;

        Ok(hwnd)
    }
}

pub(crate) fn destroy_event_sink(hwnd: HWND) {
    // SAFETY: DestroyWindow is called with the HWND created by
    // create_event_sink. The caller ensures the HWND is valid.
    unsafe {
        let _ = DestroyWindow(hwnd);
    }
}

/// WNDPROC for the event sink window.
///
/// Catches `WM_DISPLAYCHANGE` and `WM_SETTINGCHANGE` (work-area changes)
/// and forwards them as events through the thread-local sender.
pub(crate) unsafe extern "system" fn event_sink_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if let Some(event) = crate::event::translate_broadcast(msg, wparam.0) {
        EVENT_SENDER.with(|cell| {
            if let Some(sender) = cell.borrow().as_ref() {
                let _ = sender.send(event);
            }
        });
    }
    // SAFETY: DefWindowProcW is the default message handler for
    // messages not handled above. Required by the WNDPROC contract.
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}
