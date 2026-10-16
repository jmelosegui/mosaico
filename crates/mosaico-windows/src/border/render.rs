//! Painting a border's bitmap, rounded corners included, and putting it
//! on screen.

use std::mem;

use mosaico_core::{Error, Result};
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleDC, CreateDIBSection, DIB_RGB_COLORS,
    DeleteDC, DeleteObject, GetDC, ReleaseDC, SelectObject,
};
use windows::Win32::UI::WindowsAndMessaging::{ULW_ALPHA, UpdateLayeredWindow};

use crate::error::OrPlatform;

use super::{Border, Color};

/// What a failed render was doing, for its error.
const RENDER: &str = "render the border";

impl Border {
    /// Renders the border bitmap and applies it via `UpdateLayeredWindow`.
    /// Fails when the bitmap does not reach the overlay, which then keeps
    /// the one it had.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn render(
        &self,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        color: Color,
        bw: i32,
        r: i32,
    ) -> Result<()> {
        if w <= 0 || h <= 0 {
            return Err(Error::platform(RENDER, "the border has no area"));
        }

        // SAFETY: This block creates a temporary memory DC + DIB section,
        // renders the border pixels, and calls UpdateLayeredWindow to apply
        // the bitmap atomically. All GDI handles are released before return.
        unsafe {
            let screen_dc = GetDC(None);
            let mem_dc = CreateCompatibleDC(Some(screen_dc));

            let bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: w,
                    biHeight: -h, // top-down
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                },
                ..Default::default()
            };

            let mut bits = std::ptr::null_mut();
            let bmp = match CreateDIBSection(Some(mem_dc), &bmi, DIB_RGB_COLORS, &mut bits, None, 0)
            {
                Ok(bmp) => bmp,
                Err(e) => {
                    let _ = DeleteDC(mem_dc);
                    let _ = ReleaseDC(None, screen_dc);
                    return Err(Error::platform(RENDER, e));
                }
            };
            if bits.is_null() {
                let _ = DeleteObject(bmp.into());
                let _ = DeleteDC(mem_dc);
                let _ = ReleaseDC(None, screen_dc);
                return Err(Error::platform(RENDER, "the bitmap has no pixels"));
            }

            let old = SelectObject(mem_dc, bmp.into());

            // SAFETY: `bits` is non-null (checked above) and points to
            // `w * h` pixels allocated by CreateDIBSection.
            let pixel = 0xFF00_0000
                | (u32::from(color.r) << 16)
                | (u32::from(color.g) << 8)
                | u32::from(color.b);
            let buf = std::slice::from_raw_parts_mut(bits as *mut u32, (w * h) as usize);
            let iw = w - 2 * bw;
            let ih = h - 2 * bw;
            let ir = (r - bw).max(0);
            for py in 0..h {
                for px in 0..w {
                    let in_outer = in_rounded_rect(px, py, w, h, r);
                    let in_inner = in_rounded_rect(px - bw, py - bw, iw, ih, ir);
                    buf[(py * w + px) as usize] = if in_outer && !in_inner { pixel } else { 0 };
                }
            }

            let pt_dst = windows::Win32::Foundation::POINT { x, y };
            let size = windows::Win32::Foundation::SIZE { cx: w, cy: h };
            let pt_src = windows::Win32::Foundation::POINT::default();
            let blend = windows::Win32::Graphics::Gdi::BLENDFUNCTION {
                BlendOp: 0,               // AC_SRC_OVER
                SourceConstantAlpha: 255, // fully opaque border
                AlphaFormat: 1,           // AC_SRC_ALPHA
                ..Default::default()
            };

            let updated = UpdateLayeredWindow(
                self.hwnd,
                Some(screen_dc),
                Some(&pt_dst),
                Some(&size),
                Some(mem_dc),
                Some(&pt_src),
                windows::Win32::Foundation::COLORREF(0),
                Some(&blend),
                ULW_ALPHA,
            )
            .or_platform(RENDER);

            SelectObject(mem_dc, old);
            let _ = DeleteObject(bmp.into());
            let _ = DeleteDC(mem_dc);
            let _ = ReleaseDC(None, screen_dc);
            updated
        }
    }
}

/// Tests if local coordinate `(lx, ly)` is inside a rounded rectangle
/// of size `(w, h)` with corner radius `r`.
fn in_rounded_rect(lx: i32, ly: i32, w: i32, h: i32, r: i32) -> bool {
    if lx < 0 || lx >= w || ly < 0 || ly >= h {
        return false;
    }
    if r <= 0 {
        return true;
    }
    // Determine which corner zone the pixel falls in.
    let (cx, cy) = if lx < r && ly < r {
        (r, r) // top-left
    } else if lx >= w - r && ly < r {
        (w - r - 1, r) // top-right
    } else if lx < r && ly >= h - r {
        (r, h - r - 1) // bottom-left
    } else if lx >= w - r && ly >= h - r {
        (w - r - 1, h - r - 1) // bottom-right
    } else {
        return true; // not in a corner zone
    };
    let dx = lx - cx;
    let dy = ly - cy;
    dx * dx + dy * dy <= r * r
}

#[cfg(test)]
#[path = "render_tests.rs"]
mod tests;
