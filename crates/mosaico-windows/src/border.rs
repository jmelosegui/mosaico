mod class;
mod render;
mod stack;

use mosaico_core::{Rect, Result};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    CW_USEDEFAULT, CreateWindowExW, DestroyWindow, SW_HIDE, ShowWindow, WS_EX_LAYERED,
    WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT, WS_POPUP,
};
use windows::core::PCWSTR;

use crate::error::OrPlatform;

use class::{CLASS_NAME, ensure_class_registered};

/// The color type borders and the bar draw with, which lives in core.
pub(crate) use mosaico_core::config::Color;

/// A transparent overlay window that displays a colored border.
///
/// Uses `UpdateLayeredWindow` with per-pixel alpha so no message pump
/// is needed on the creating thread.
pub struct Border {
    hwnd: HWND,
}

impl Border {
    /// Creates a new hidden border overlay window.
    pub fn new() -> Result<Self> {
        ensure_class_registered();

        let ex = WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TRANSPARENT;
        // SAFETY: CreateWindowExW creates a layered popup window using the
        // class registered above. The class name pointer is valid for the
        // lifetime of the process.
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
            )
        }
        .or_platform("create a border window")?;

        Ok(Self { hwnd })
    }

    /// Shows the border around the given rectangle with the specified color.
    ///
    /// `target` is the HWND of the window the border surrounds, focused
    /// or not. The border is placed just
    /// behind it in z-order and inherits its topmost state so it stays
    /// visible with topmost windows without flickering for normal ones.
    ///
    /// Renders the bitmap before making the window visible to avoid
    /// flickering caused by a stale bitmap being shown at the new size.
    /// Fails when the new bitmap does not reach the overlay; the overlay
    /// is put in place either way.
    pub(crate) fn show(
        &self,
        rect: &Rect,
        color: Color,
        width: i32,
        radius: i32,
        target: HWND,
    ) -> Result<()> {
        let w = rect.width + width * 2;
        let h = rect.height + width * 2;

        // Render the new bitmap first so `UpdateLayeredWindow` atomically
        // sets position, size, and pixel content in one call.
        let rendered = self.render(rect.x - width, rect.y - width, w, h, color, width, radius);
        self.restack(rect, width, target);
        rendered
    }

    /// Hides the border.
    pub fn hide(&self) {
        // SAFETY: ShowWindow hides the border overlay owned by this struct.
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_HIDE);
        }
    }
}

impl Drop for Border {
    fn drop(&mut self) {
        // SAFETY: DestroyWindow cleans up the border overlay. The HWND
        // is exclusively owned by this struct and not used after drop.
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

#[cfg(test)]
#[path = "border/show_tests.rs"]
mod tests;
