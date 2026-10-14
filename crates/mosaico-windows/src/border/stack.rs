//! Putting a border overlay just behind the window it surrounds.

use mosaico_core::Rect;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GWL_EXSTYLE, GetWindowLongPtrW, HWND_NOTOPMOST, HWND_TOPMOST, SWP_NOACTIVATE, SWP_SHOWWINDOW,
    SetWindowPos, WS_EX_TOPMOST,
};

use super::Border;

impl Border {
    /// Puts the border just behind `target` and shows it, around `rect`
    /// with a stroke `width` wide, with the bitmap it has.
    ///
    /// A border drawn again the same way needs only this: the window it
    /// surrounds may have moved in the z-order, or the border may have
    /// been hidden.
    pub(crate) fn restack(&self, rect: &Rect, width: i32, target: HWND) {
        let w = rect.width + width * 2;
        let h = rect.height + width * 2;

        // Match the target window's topmost state, then place the border
        // just behind it so the border is visible around the window but
        // never covers it or other windows that should be in front.
        // SAFETY: GetWindowLongPtrW reads the extended style of the target
        // window to determine whether it is topmost. The HWND is provided
        // by the caller (the focused window).
        let target_topmost = unsafe {
            let ex = GetWindowLongPtrW(target, GWL_EXSTYLE) as u32;
            (ex & WS_EX_TOPMOST.0) != 0
        };
        let insert_after = if target_topmost {
            HWND_TOPMOST
        } else {
            HWND_NOTOPMOST
        };

        // SAFETY: SetWindowPos repositions the border overlay. The border
        // HWND is owned by this struct and the target HWND is the caller's
        // focused window.
        unsafe {
            // First set the topmost/non-topmost band.
            let _ = SetWindowPos(
                self.hwnd,
                Some(insert_after),
                0,
                0,
                0,
                0,
                SWP_NOACTIVATE
                    | SWP_SHOWWINDOW
                    | windows::Win32::UI::WindowsAndMessaging::SWP_NOMOVE
                    | windows::Win32::UI::WindowsAndMessaging::SWP_NOSIZE,
            );
            // Then position just behind the target window.
            let _ = SetWindowPos(
                self.hwnd,
                Some(target),
                rect.x - width,
                rect.y - width,
                w,
                h,
                SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );
        }
    }
}
