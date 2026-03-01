use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassW, SPI_SETWORKAREA,
    WM_DISPLAYCHANGE, WM_SETTINGCHANGE, WNDCLASSW, WS_EX_TOOLWINDOW,
};
use windows::core::w;

use mosaico_core::WindowEvent;

use super::EVENT_SENDER;

/// Creates a hidden window that receives `WM_DISPLAYCHANGE`.
///
/// Must NOT be a message-only window (`HWND_MESSAGE` parent) because those
/// do not receive broadcast messages. Instead we create a regular hidden
/// window with `WS_EX_TOOLWINDOW` to keep it out of the taskbar.
pub(crate) fn create_event_sink() -> Option<HWND> {
    unsafe {
        let class_name = w!("MosaicoEventSink");
        let wc = WNDCLASSW {
            lpfnWndProc: Some(event_sink_proc),
            lpszClassName: class_name,
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

pub(crate) fn destroy_event_sink(hwnd: HWND) {
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
    if msg == WM_DISPLAYCHANGE {
        EVENT_SENDER.with(|cell| {
            if let Some(sender) = cell.borrow().as_ref() {
                let _ = sender.send(WindowEvent::DisplayChanged);
            }
        });
    } else if msg == WM_SETTINGCHANGE && wparam.0 == SPI_SETWORKAREA.0 as usize {
        EVENT_SENDER.with(|cell| {
            if let Some(sender) = cell.borrow().as_ref() {
                let _ = sender.send(WindowEvent::WorkAreaChanged);
            }
        });
    }
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}
