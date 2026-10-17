//! What a bar is painted on before it goes to the screen: a 32-bit DIB
//! in a memory device context.

use std::mem;

use mosaico_core::{Error, Result};
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleDC, CreateDIBSection, DIB_RGB_COLORS,
    DeleteDC, DeleteObject, GetDC, HBITMAP, HDC, HGDIOBJ, ReleaseDC, SelectObject,
};

use crate::error::OrPlatform;

/// A `w` by `h` top-down BGRA bitmap selected into a memory DC that is
/// compatible with the screen. Dropping it releases all three.
pub(super) struct Canvas {
    pub(super) screen_dc: HDC,
    pub(super) dc: HDC,
    bitmap: HBITMAP,
    old: HGDIOBJ,
    bits: *mut u32,
    pub(super) w: i32,
    pub(super) h: i32,
}

impl Canvas {
    /// A canvas `w` by `h` pixels, both above 0.
    pub(super) fn new(w: i32, h: i32) -> Result<Self> {
        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w,
                biHeight: -h,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        // SAFETY: the screen DC is only read from, the memory DC and the
        // DIB section are made for this canvas, and every handle made is
        // released here on failure or when the canvas drops.
        unsafe {
            let screen_dc = GetDC(None);
            let dc = CreateCompatibleDC(Some(screen_dc));
            let mut bits = std::ptr::null_mut();
            let created = CreateDIBSection(Some(dc), &bmi, DIB_RGB_COLORS, &mut bits, None, 0)
                .or_platform("create the bar bitmap")
                .and_then(|bitmap| {
                    if bits.is_null() {
                        let _ = DeleteObject(bitmap.into());
                        Err(Error::platform(
                            "create the bar bitmap",
                            "the bitmap has no pixels",
                        ))
                    } else {
                        Ok(bitmap)
                    }
                });
            let bitmap = match created {
                Ok(bitmap) => bitmap,
                Err(e) => {
                    let _ = DeleteDC(dc);
                    let _ = ReleaseDC(None, screen_dc);
                    return Err(e);
                }
            };
            let old = SelectObject(dc, bitmap.into());
            Ok(Self {
                screen_dc,
                dc,
                bitmap,
                old,
                bits: bits.cast(),
                w,
                h,
            })
        }
    }

    /// The pixels, row by row from the top.
    pub(super) fn pixels(&mut self) -> &mut [u32] {
        // SAFETY: CreateDIBSection allocated `w * h` pixels at `bits`,
        // which stay valid until the bitmap is deleted, when the canvas
        // drops. The borrow of the canvas keeps any other use out.
        unsafe { std::slice::from_raw_parts_mut(self.bits, (self.w * self.h) as usize) }
    }
}

impl Drop for Canvas {
    fn drop(&mut self) {
        // SAFETY: puts back the bitmap the memory DC had, then releases
        // the handles the canvas made.
        unsafe {
            SelectObject(self.dc, self.old);
            let _ = DeleteObject(self.bitmap.into());
            let _ = DeleteDC(self.dc);
            let _ = ReleaseDC(None, self.screen_dc);
        }
    }
}
