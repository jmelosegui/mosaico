//! Putting a border overlay just behind the window it surrounds, or
//! only moving it.

use mosaico_core::Rect;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GWL_EXSTYLE, GetWindowLongPtrW, HWND_NOTOPMOST, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE,
    SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW, SetWindowPos, WS_EX_TOPMOST,
};

use super::Border;

impl Border {
    /// Moves the border so it surrounds `rect` with a stroke `width`
    /// wide, and changes nothing else: its size, its bitmap, its place in
    /// the z-order and whether it shows stay as they are.
    pub(crate) fn move_to(&self, rect: &Rect, width: i32) {
        let flags = SWP_NOACTIVATE | SWP_NOZORDER | SWP_NOSIZE;
        // SAFETY: SetWindowPos only moves the border overlay, which this
        // struct owns.
        unsafe {
            let _ = SetWindowPos(self.hwnd, None, rect.x - width, rect.y - width, 0, 0, flags);
        }
    }

    /// Puts the border just behind `target` and shows it, around `rect`
    /// with a stroke `width` wide, with the bitmap it has.
    ///
    /// A border drawn again with the same picture needs only this: the
    /// window it surrounds may have moved in the z-order or to another
    /// slot of the same size, or the border may have been hidden. The
    /// border is shown only once it is around `rect`, behind `target`, so
    /// a hidden one never shows for an instant where it was hidden.
    pub(crate) fn restack(&self, rect: &Rect, width: i32, target: HWND) {
        // Match the target window's topmost state, then place the border
        // just behind it so the border is visible around the window but
        // never covers it or other windows that should be in front.
        self.join_band_of(target);
        self.place_behind(rect, width, target);
    }

    /// Puts the border in the topmost band when `target` is topmost, and
    /// out of it when it is not. The border stays where it is, and hidden
    /// if it was.
    pub(super) fn join_band_of(&self, target: HWND) {
        // SAFETY: GetWindowLongPtrW reads the extended style of the target
        // window to determine whether it is topmost. The HWND is provided
        // by the caller: the window the border surrounds.
        let target_topmost = unsafe {
            let ex = GetWindowLongPtrW(target, GWL_EXSTYLE) as u32;
            (ex & WS_EX_TOPMOST.0) != 0
        };
        let band = if target_topmost {
            HWND_TOPMOST
        } else {
            HWND_NOTOPMOST
        };
        // SAFETY: SetWindowPos changes only the z-order band of the border
        // overlay, which this struct owns.
        unsafe {
            let _ = SetWindowPos(
                self.hwnd,
                Some(band),
                0,
                0,
                0,
                0,
                SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE,
            );
        }
    }

    /// Moves the border around `rect`, just behind `target`, and shows it.
    pub(super) fn place_behind(&self, rect: &Rect, width: i32, target: HWND) {
        let w = rect.width + width * 2;
        let h = rect.height + width * 2;
        // SAFETY: SetWindowPos repositions the border overlay. The border
        // HWND is owned by this struct and the target HWND is the window
        // the border surrounds.
        unsafe {
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
