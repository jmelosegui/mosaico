//! Moving a shown window into place, and repairing how it draws.

use mosaico_core::Result;
use windows::Win32::UI::WindowsAndMessaging::{
    SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOCOPYBITS, SWP_NOSENDCHANGING, SWP_NOZORDER,
    SetWindowPos,
};

use crate::error::OrPlatform;
use crate::frame;

use super::Window;

impl Window {
    /// Returns whether this window needs `SWP_FRAMECHANGED` to update
    /// its rendering surface after a programmatic resize.
    ///
    /// Chromium-based apps (Chrome, Edge, Electron) use a GPU compositor
    /// that only repaints when `WM_NCCALCSIZE` fires. Without
    /// `SWP_FRAMECHANGED`, they render a blank/stale surface.
    fn needs_frame_changed(&self) -> bool {
        let class = self.class().unwrap_or_default();
        class == "Chrome_WidgetWin_1" || class == "MozillaWindowClass"
    }

    /// Moves and resizes the window so its visible bounds are `rect`.
    pub(crate) fn set_rect(&self, rect: &mosaico_core::Rect) -> Result<()> {
        // If the window is maximized, clear the WS_MAXIMIZE style first.
        // SetWindowPos alone doesn't remove the style, leaving the window
        // in a "maximized but tiled-sized" ghost state with wrong frame
        // metrics (e.g. extra top border offset).
        if self.is_maximized() {
            use windows::Win32::UI::WindowsAndMessaging::{
                GWL_STYLE, GetWindowLongW, SetWindowLongW, WS_MAXIMIZE,
            };
            // SAFETY: GetWindowLongW/SetWindowLongW clear WS_MAXIMIZE on a valid HWND.
            unsafe {
                let style = GetWindowLongW(self.hwnd, GWL_STYLE);
                SetWindowLongW(self.hwnd, GWL_STYLE, style & !(WS_MAXIMIZE.0 as i32));
            }
        }

        // Compensate for invisible borders so the visible portion
        // lands exactly at the requested position and size.
        let border = frame::border_offset(self.hwnd)?;

        let x = rect.x - border.left;
        let y = rect.y - border.top;
        let cx = rect.width + border.left + border.right;
        let cy = rect.height + border.top + border.bottom;

        // SWP_NOSENDCHANGING suppresses WM_WINDOWPOSCHANGING so the
        // window cannot reject or modify the requested dimensions.
        //
        // SWP_NOCOPYBITS discards old client-area contents instead
        // of blitting them, avoiding stale surfaces.
        let mut flags = SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOSENDCHANGING | SWP_NOCOPYBITS;

        // GPU-composited apps (Chrome, Electron) need SWP_FRAMECHANGED
        // to trigger WM_NCCALCSIZE, which makes them recalculate their
        // client area and update the rendering surface. However,
        // SWP_FRAMECHANGED also lets windows enforce size constraints
        // via WM_NCCALCSIZE, so we only add it for Chromium-based
        // windows that are known to need it.
        let frame_changed = self.needs_frame_changed();
        if frame_changed {
            flags |= SWP_FRAMECHANGED;
        }

        let hwnd_val = self.hwnd.0 as usize;
        mosaico_core::log_debug!(
            "set_rect 0x{:X}: target({},{} {}x{}) border(L:{} T:{} R:{} B:{}) frame_changed={}",
            hwnd_val,
            rect.x,
            rect.y,
            rect.width,
            rect.height,
            border.left,
            border.top,
            border.right,
            border.bottom,
            frame_changed
        );

        // SAFETY: SetWindowPos with a valid HWND is safe.
        unsafe { SetWindowPos(self.hwnd, None, x, y, cx, cy, flags) }.or_platform("move the window")
    }

    /// Repaints the window, its frame and its children, and puts the GPU
    /// surface of a Chromium window back in place if it drifted.
    ///
    /// Fails when the redraw does; the realignment is tried either way.
    pub(crate) fn invalidate(&self) -> Result<()> {
        use windows::Win32::Graphics::Gdi::{
            RDW_ALLCHILDREN, RDW_ERASE, RDW_FRAME, RDW_INVALIDATE, RDW_UPDATENOW, RedrawWindow,
        };

        // SAFETY: RedrawWindow forces a complete repaint of the window,
        // its frame, and all child windows. The aggressive flag
        // combination ensures GPU-composited apps (Chrome, Electron)
        // refresh their rendering surface after a programmatic resize.
        let redrawn = unsafe {
            RedrawWindow(
                Some(self.hwnd),
                None,
                None,
                RDW_INVALIDATE | RDW_UPDATENOW | RDW_FRAME | RDW_ERASE | RDW_ALLCHILDREN,
            )
        };
        if self.needs_frame_changed() {
            crate::chromium::realign_gpu_child(self.hwnd);
        }
        redrawn.ok().or_platform("repaint the window")
    }
}
