//! Status bar overlay windows — one per monitor.
//!
//! Each bar is a layered Win32 popup that renders workspace indicators,
//! layout info, and system widgets using `UpdateLayeredWindow`.

mod render;
mod text;
pub(crate) mod widgets;

use std::sync::Once;

use mosaico_core::Rect;
use mosaico_core::config::bar::BarConfig;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, DestroyWindow, HWND_TOP, RegisterClassW,
    SW_HIDE, SWP_NOACTIVATE, SWP_SHOWWINDOW, SetWindowPos, ShowWindow, WNDCLASSW, WS_EX_LAYERED,
    WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT, WS_POPUP,
};
use windows::core::PCWSTR;

use render::render_bar;
pub use widgets::BarState;

/// A status bar overlay for a single monitor.
pub struct Bar {
    hwnd: HWND,
    monitor_rect: Rect,
}

static REGISTER_CLASS: Once = Once::new();
const CLASS_NAME: &[u16] = &[
    b'M' as u16,
    b'o' as u16,
    b's' as u16,
    b'a' as u16,
    b'i' as u16,
    b'c' as u16,
    b'o' as u16,
    b'B' as u16,
    b'a' as u16,
    b'r' as u16,
    0,
];

fn ensure_class_registered() {
    REGISTER_CLASS.call_once(|| {
        let wc = WNDCLASSW {
            lpfnWndProc: Some(bar_wnd_proc),
            lpszClassName: PCWSTR(CLASS_NAME.as_ptr()),
            ..Default::default()
        };
        unsafe {
            RegisterClassW(&wc);
        }
    });
}

unsafe extern "system" fn bar_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

impl Bar {
    /// Creates a new hidden bar overlay positioned at the top of the
    /// given monitor work area.
    pub fn new(monitor_work_area: Rect) -> Result<Self, Box<dyn std::error::Error>> {
        ensure_class_registered();

        let ex = WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TRANSPARENT;
        let hwnd = unsafe {
            CreateWindowExW(
                ex,
                PCWSTR(CLASS_NAME.as_ptr()),
                PCWSTR::null(),
                WS_POPUP,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                0,
                0,
                None,
                None,
                None,
                None,
            )?
        };

        Ok(Self {
            hwnd,
            monitor_rect: monitor_work_area,
        })
    }

    /// Renders and shows the bar with the given config and state.
    pub fn update(&self, config: &BarConfig, state: &BarState) {
        let x = self.monitor_rect.x;
        let y = self.monitor_rect.y;
        let w = self.monitor_rect.width;
        let h = config.height;

        render_bar(self.hwnd, x, y, w, h, config, state);

        unsafe {
            let _ = SetWindowPos(
                self.hwnd,
                Some(HWND_TOP),
                x,
                y,
                w,
                h,
                SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );
        }
    }

    /// Hides the bar.
    pub fn hide(&self) {
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_HIDE);
        }
    }
}

impl Drop for Bar {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
    }
}
