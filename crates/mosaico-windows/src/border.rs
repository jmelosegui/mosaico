use std::mem;
use std::sync::Once;

use mosaico_core::{Rect, WindowResult};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleDC, CreateDIBSection, DIB_RGB_COLORS,
    DeleteDC, DeleteObject, GetDC, ReleaseDC, SelectObject,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, DestroyWindow, HWND_TOPMOST, RegisterClassW,
    SW_HIDE, SWP_NOACTIVATE, SWP_SHOWWINDOW, SetWindowPos, ShowWindow, ULW_ALPHA,
    UpdateLayeredWindow, WNDCLASSW, WS_EX_LAYERED, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
    WS_EX_TRANSPARENT, WS_POPUP,
};
use windows::core::PCWSTR;

/// RGB color parsed from a hex string.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    /// Parses a hex color string like "#00b4d8" or "00b4d8".
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }
        Some(Self {
            r: u8::from_str_radix(&hex[0..2], 16).ok()?,
            g: u8::from_str_radix(&hex[2..4], 16).ok()?,
            b: u8::from_str_radix(&hex[4..6], 16).ok()?,
        })
    }
}

/// A transparent overlay window that displays a colored border.
///
/// Uses `UpdateLayeredWindow` with per-pixel alpha so no message pump
/// is needed on the creating thread.
pub struct Border {
    hwnd: HWND,
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
    b'o' as u16,
    b'r' as u16,
    b'd' as u16,
    b'e' as u16,
    b'r' as u16,
    0,
];

fn ensure_class_registered() {
    REGISTER_CLASS.call_once(|| {
        let wc = WNDCLASSW {
            lpfnWndProc: Some(border_wnd_proc),
            lpszClassName: PCWSTR(CLASS_NAME.as_ptr()),
            ..Default::default()
        };
        unsafe {
            RegisterClassW(&wc);
        }
    });
}

unsafe extern "system" fn border_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

impl Border {
    /// Creates a new hidden border overlay window.
    pub fn new() -> WindowResult<Self> {
        ensure_class_registered();

        let ex = WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_TRANSPARENT;
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

        Ok(Self { hwnd })
    }

    /// Shows the border around the given rectangle with the specified color.
    pub fn show(&self, rect: &Rect, color: Color, width: i32) {
        let w = rect.width + width * 2;
        let h = rect.height + width * 2;

        unsafe {
            let _ = SetWindowPos(
                self.hwnd,
                Some(HWND_TOPMOST),
                rect.x - width,
                rect.y - width,
                w,
                h,
                SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );
        }

        self.render(rect.x - width, rect.y - width, w, h, color, width);
    }

    /// Hides the border.
    pub fn hide(&self) {
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_HIDE);
        }
    }

    /// Renders the border bitmap and applies it via `UpdateLayeredWindow`.
    fn render(&self, x: i32, y: i32, w: i32, h: i32, color: Color, bw: i32) {
        if w <= 0 || h <= 0 {
            return;
        }

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
            let Ok(bmp) = CreateDIBSection(Some(mem_dc), &bmi, DIB_RGB_COLORS, &mut bits, None, 0)
            else {
                let _ = DeleteDC(mem_dc);
                let _ = ReleaseDC(None, screen_dc);
                return;
            };
            if bits.is_null() {
                let _ = DeleteObject(bmp.into());
                let _ = DeleteDC(mem_dc);
                let _ = ReleaseDC(None, screen_dc);
                return;
            }

            let old = SelectObject(mem_dc, bmp.into());

            // SAFETY: `bits` is non-null (checked above) and points to
            // `w * h` pixels allocated by CreateDIBSection.
            let pixel = 0xFF00_0000
                | (u32::from(color.r) << 16)
                | (u32::from(color.g) << 8)
                | u32::from(color.b);
            let buf = std::slice::from_raw_parts_mut(bits as *mut u32, (w * h) as usize);
            for py in 0..h {
                for px in 0..w {
                    let on_border = px < bw || px >= w - bw || py < bw || py >= h - bw;
                    buf[(py * w + px) as usize] = if on_border { pixel } else { 0 };
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

            let _ = UpdateLayeredWindow(
                self.hwnd,
                Some(screen_dc),
                Some(&pt_dst),
                Some(&size),
                Some(mem_dc),
                Some(&pt_src),
                windows::Win32::Foundation::COLORREF(0),
                Some(&blend),
                ULW_ALPHA,
            );

            SelectObject(mem_dc, old);
            let _ = DeleteObject(bmp.into());
            let _ = DeleteDC(mem_dc);
            let _ = ReleaseDC(None, screen_dc);
        }
    }
}

impl Drop for Border {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_color() {
        // Assert
        assert_eq!(
            Color::from_hex("#00b4d8"),
            Some(Color {
                r: 0x00,
                g: 0xB4,
                b: 0xD8
            })
        );
        assert_eq!(
            Color::from_hex("2d6a4f"),
            Some(Color {
                r: 0x2D,
                g: 0x6A,
                b: 0x4F
            })
        );
        assert_eq!(Color::from_hex("bad"), None);
    }
}
