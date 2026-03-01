use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, GA_ROOT, GetAncestor, GetCursorPos, KillTimer,
    PostMessageW, RegisterClassW, SPI_SETWORKAREA, SetTimer, WM_APP, WM_DISPLAYCHANGE,
    WM_SETTINGCHANGE, WM_TIMER, WNDCLASSW, WS_EX_TOOLWINDOW, WindowFromPoint,
};
use windows::core::w;

use mosaico_core::WindowEvent;

use super::EVENT_SENDER;

const FOCUS_FOLLOWS_MOUSE_TIMER_ID: usize = 0x4D4F;
const FOCUS_FOLLOWS_MOUSE_TOGGLE_MSG: u32 = WM_APP + 1;
const FOCUS_FOLLOWS_MOUSE_INTERVAL_MS: u32 = 100;
const FOCUS_FOLLOWS_MOUSE_DEBOUNCE_TICKS: u8 = 3;

#[derive(Clone, Copy)]
struct HoverState {
    last_hwnd: usize,
    stable_ticks: u8,
    last_sent: usize,
}

impl HoverState {
    fn reset(&mut self) {
        self.last_hwnd = 0;
        self.stable_ticks = 0;
        self.last_sent = 0;
    }
}

thread_local! {
    static HOVER_STATE: std::cell::RefCell<HoverState> = const { std::cell::RefCell::new(HoverState {
        last_hwnd: 0,
        stable_ticks: 0,
        last_sent: 0,
    }) };
}

/// Creates a hidden window that receives `WM_DISPLAYCHANGE` and
/// focus-follows-mouse timer ticks.
///
/// Must NOT be a message-only window (`HWND_MESSAGE` parent) because those
/// do not receive broadcast messages. Instead we create a regular hidden
/// window with `WS_EX_TOOLWINDOW` to keep it out of the taskbar.
pub(crate) fn create_event_sink(focus_follows_mouse: bool) -> Option<HWND> {
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
            Ok(h) if !h.is_invalid() => {
                if focus_follows_mouse {
                    let _ = SetTimer(
                        Some(h),
                        FOCUS_FOLLOWS_MOUSE_TIMER_ID,
                        FOCUS_FOLLOWS_MOUSE_INTERVAL_MS,
                        None,
                    );
                }
                Some(h)
            }
            _ => {
                eprintln!("Failed to create MosaicoEventSink window");
                None
            }
        }
    }
}

pub(crate) fn destroy_event_sink(hwnd: HWND, focus_follows_mouse: bool) {
    unsafe {
        if focus_follows_mouse {
            let _ = KillTimer(Some(hwnd), FOCUS_FOLLOWS_MOUSE_TIMER_ID);
        }
        let _ = DestroyWindow(hwnd);
    }
}

pub(crate) fn toggle_focus_follows_mouse(hwnd: HWND, enabled: bool) {
    let flag = if enabled { 1 } else { 0 };
    unsafe {
        let _ = PostMessageW(
            Some(hwnd),
            FOCUS_FOLLOWS_MOUSE_TOGGLE_MSG,
            WPARAM(flag),
            LPARAM(0),
        );
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
    } else if msg == FOCUS_FOLLOWS_MOUSE_TOGGLE_MSG {
        if wparam.0 == 1 {
            unsafe {
                let _ = SetTimer(
                    Some(hwnd),
                    FOCUS_FOLLOWS_MOUSE_TIMER_ID,
                    FOCUS_FOLLOWS_MOUSE_INTERVAL_MS,
                    None,
                );
            }
        } else {
            unsafe {
                let _ = KillTimer(Some(hwnd), FOCUS_FOLLOWS_MOUSE_TIMER_ID);
            }
            HOVER_STATE.with(|cell| cell.borrow_mut().reset());
        }
    } else if msg == WM_TIMER && wparam.0 == FOCUS_FOLLOWS_MOUSE_TIMER_ID {
        let mut point = POINT::default();
        if unsafe { GetCursorPos(&mut point) }.is_ok() {
            let hwnd = unsafe { WindowFromPoint(point) };
            let root = unsafe { GetAncestor(hwnd, GA_ROOT) };
            let root_handle = root.0 as usize;
            if root_handle != 0 {
                HOVER_STATE.with(|cell| {
                    let mut state = cell.borrow_mut();
                    if state.last_hwnd == root_handle {
                        state.stable_ticks = state.stable_ticks.saturating_add(1);
                    } else {
                        state.last_hwnd = root_handle;
                        state.stable_ticks = 1;
                        state.last_sent = 0;
                    }
                    if state.stable_ticks >= FOCUS_FOLLOWS_MOUSE_DEBOUNCE_TICKS
                        && state.last_sent != root_handle
                    {
                        state.last_sent = root_handle;
                        EVENT_SENDER.with(|cell| {
                            if let Some(sender) = cell.borrow().as_ref() {
                                let _ = sender.send(WindowEvent::MouseHover { hwnd: root_handle });
                            }
                        });
                    }
                });
            }
        }
    }
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}
