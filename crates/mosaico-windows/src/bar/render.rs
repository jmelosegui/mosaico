//! DIB-based rendering orchestrator for the status bar.
//!
//! Sets up the 32-bit BGRA bitmap and font, fills the background,
//! delegates widget drawing, then applies via `UpdateLayeredWindow`.

use std::mem;

use mosaico_core::config::bar::BarConfig;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleDC, CreateDIBSection, CreateFontW,
    DIB_RGB_COLORS, DeleteDC, DeleteObject, FONT_CHARSET, FONT_CLIP_PRECISION,
    FONT_OUTPUT_PRECISION, FONT_QUALITY, GetDC, ReleaseDC, SelectObject, SetBkMode, TRANSPARENT,
};
use windows::Win32::UI::WindowsAndMessaging::{ULW_ALPHA, UpdateLayeredWindow};
use windows::core::PCWSTR;

use super::text::{DrawCtx, pixel_from_color_alpha};
use super::widgets::{self, BarState};
use crate::border::Color;

/// Renders the full bar content and applies it to the layered window.
pub fn render_bar(
    hwnd: HWND,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    config: &BarConfig,
    state: &BarState,
) {
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
                biHeight: -h,
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

        let old_bmp = SelectObject(mem_dc, bmp.into());

        // Fill background (with opacity for transparency support)
        let bg = Color::from_hex(&config.colors.background).unwrap_or(Color {
            r: 0x1e,
            g: 0x1e,
            b: 0x2e,
        });
        let alpha = (config.background_opacity * 255 / 100) as u8;
        let bg_pixel = pixel_from_color_alpha(bg, alpha);
        let buf = std::slice::from_raw_parts_mut(bits as *mut u32, (w * h) as usize);
        buf.fill(bg_pixel);

        // Create font from config
        let font_wide: Vec<u16> = config
            .font
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let font = CreateFontW(
            config.font_size,
            0,
            0,
            0,
            400,
            0,
            0,
            0,
            FONT_CHARSET(0),
            FONT_OUTPUT_PRECISION(0),
            FONT_CLIP_PRECISION(0),
            FONT_QUALITY(0),
            0,
            PCWSTR(font_wide.as_ptr()),
        );
        let bold_font = CreateFontW(
            config.font_size,
            0,
            0,
            0,
            700,
            0,
            0,
            0,
            FONT_CHARSET(0),
            FONT_OUTPUT_PRECISION(0),
            FONT_CLIP_PRECISION(0),
            FONT_QUALITY(0),
            0,
            PCWSTR(font_wide.as_ptr()),
        );
        let old_font = SelectObject(mem_dc, font.into());
        let _ = SetBkMode(mem_dc, TRANSPARENT);

        let mut ctx = DrawCtx {
            dc: mem_dc,
            buf,
            w,
            h,
            bg_pixel,
            font: font.into(),
            bold_font: bold_font.into(),
        };

        // Delegate to widget renderers
        widgets::draw_left(&mut ctx, config, state);
        widgets::draw_right(&mut ctx, config, state);

        // Apply to layered window
        apply_layered(hwnd, screen_dc, mem_dc, x, y, w, h);

        // Cleanup
        SelectObject(mem_dc, old_font);
        let _ = DeleteObject(font.into());
        let _ = DeleteObject(bold_font.into());
        SelectObject(mem_dc, old_bmp);
        let _ = DeleteObject(bmp.into());
        let _ = DeleteDC(mem_dc);
        let _ = ReleaseDC(None, screen_dc);
    }
}

/// Calls `UpdateLayeredWindow` to atomically present the bitmap.
unsafe fn apply_layered(
    hwnd: HWND,
    screen_dc: windows::Win32::Graphics::Gdi::HDC,
    mem_dc: windows::Win32::Graphics::Gdi::HDC,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
) {
    let blend = windows::Win32::Graphics::Gdi::BLENDFUNCTION {
        BlendOp: 0,
        SourceConstantAlpha: 255,
        AlphaFormat: 1,
        ..Default::default()
    };
    unsafe {
        let _ = UpdateLayeredWindow(
            hwnd,
            Some(screen_dc),
            Some(&windows::Win32::Foundation::POINT { x, y }),
            Some(&windows::Win32::Foundation::SIZE { cx: w, cy: h }),
            Some(mem_dc),
            Some(&windows::Win32::Foundation::POINT::default()),
            windows::Win32::Foundation::COLORREF(0),
            Some(&blend),
            ULW_ALPHA,
        );
    }
}
