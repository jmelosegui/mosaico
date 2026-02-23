//! Low-level text and shape rendering helpers for the bar.
//!
//! Handles GDI text output onto a DIB, rounded-rect pill backgrounds,
//! and the alpha-fix needed for `UpdateLayeredWindow` compatibility.

use windows::Win32::Graphics::Gdi::{GetTextExtentPoint32W, HDC, SetTextColor, TextOutW};

use crate::border::Color;

/// Bundles the GDI DC, pixel buffer, and dimensions needed by all
/// draw helpers, keeping argument lists short.
pub struct DrawCtx<'a> {
    pub dc: HDC,
    pub buf: &'a mut [u32],
    pub w: i32,
    pub h: i32,
    pub bg_pixel: u32,
}

/// Draws a text string at (x, vertically centered) and returns the X
/// position after the text.
pub fn draw_text(ctx: &mut DrawCtx, x: i32, text: &str, color_hex: &str) -> i32 {
    if text.is_empty() {
        return x;
    }

    let color = Color::from_hex(color_hex).unwrap_or(Color {
        r: 0xcd,
        g: 0xd6,
        b: 0xf4,
    });

    let wide: Vec<u16> = text.encode_utf16().collect();
    let text_w = measure_text_wide(ctx.dc, &wide);

    let mut text_size = windows::Win32::Foundation::SIZE::default();
    unsafe {
        let _ = GetTextExtentPoint32W(ctx.dc, &wide, &mut text_size);
    }
    let y = (ctx.h - text_size.cy) / 2;

    unsafe {
        let _ = SetTextColor(
            ctx.dc,
            windows::Win32::Foundation::COLORREF(
                u32::from(color.r) | (u32::from(color.g) << 8) | (u32::from(color.b) << 16),
            ),
        );
        let _ = TextOutW(ctx.dc, x, y, &wide);
    }

    fix_alpha_region(ctx, x, y, text_w, text_size.cy);
    x + text_w
}

/// Measures text width in pixels.
pub fn measure_text(dc: HDC, text: &str) -> i32 {
    let wide: Vec<u16> = text.encode_utf16().collect();
    measure_text_wide(dc, &wide)
}

/// Measures pre-encoded UTF-16 text width.
fn measure_text_wide(dc: HDC, wide: &[u16]) -> i32 {
    let mut size = windows::Win32::Foundation::SIZE::default();
    unsafe {
        let _ = GetTextExtentPoint32W(dc, wide, &mut size);
    }
    size.cx
}

/// Draws a rounded-rectangle pill background with optional border.
///
/// `radius` controls corner rounding (0 = sharp corners).
/// `border_hex` draws a border of `border_width` pixels if non-empty.
#[allow(clippy::too_many_arguments)]
pub fn draw_pill(
    ctx: &mut DrawCtx,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    color_hex: &str,
    radius: i32,
    border_hex: &str,
    border_width: i32,
) {
    let fill = Color::from_hex(color_hex).unwrap_or(Color {
        r: 0x31,
        g: 0x32,
        b: 0x44,
    });
    let fill_px = pixel_from_color(fill);
    let border_px = Color::from_hex(border_hex).map(pixel_from_color);
    let bw = border_width.max(0);
    let r = radius.min(w / 2).min(h / 2);

    let x0 = x.max(0);
    let y0 = y.max(0);
    let x1 = (x + w).min(ctx.w);
    let y1 = (y + h).min(ctx.h);

    // Inner rect for border detection: inset by border_width on each side.
    let iw = w - 2 * bw;
    let ih = h - 2 * bw;
    let ir = (r - bw).max(0);

    for py in y0..y1 {
        for px in x0..x1 {
            let lx = px - x;
            let ly = py - y;
            if !in_rounded_rect(lx, ly, w, h, r) {
                continue;
            }
            let idx = (py * ctx.w + px) as usize;
            if idx >= ctx.buf.len() {
                continue;
            }
            // Border pixel: inside outer shape but outside the inner inset.
            if let Some(bp) = border_px
                && !in_rounded_rect(lx - bw, ly - bw, iw, ih, ir)
            {
                ctx.buf[idx] = bp;
                continue;
            }
            ctx.buf[idx] = fill_px;
        }
    }
}

/// Tests if a local coordinate (lx, ly) is inside a rounded rect of
/// size (w, h) with corner radius r.
fn in_rounded_rect(lx: i32, ly: i32, w: i32, h: i32, r: i32) -> bool {
    if lx < 0 || lx >= w || ly < 0 || ly >= h {
        return false;
    }
    if r <= 0 {
        return true;
    }
    // Check four corners
    let (cx, cy) = if lx < r && ly < r {
        (r, r) // top-left
    } else if lx >= w - r && ly < r {
        (w - r - 1, r) // top-right
    } else if lx < r && ly >= h - r {
        (r, h - r - 1) // bottom-left
    } else if lx >= w - r && ly >= h - r {
        (w - r - 1, h - r - 1) // bottom-right
    } else {
        return true; // not in a corner
    };
    let dx = lx - cx;
    let dy = ly - cy;
    dx * dx + dy * dy <= r * r
}

/// Converts an RGB Color to a fully-opaque BGRA pixel value.
pub fn pixel_from_color(c: Color) -> u32 {
    0xFF00_0000 | (u32::from(c.r) << 16) | (u32::from(c.g) << 8) | u32::from(c.b)
}

/// Converts an RGB Color with a separate alpha (0–255) to a
/// premultiplied-alpha BGRA pixel value.
///
/// `UpdateLayeredWindow` with `AC_SRC_ALPHA` requires premultiplied
/// pixels: each channel is scaled by `alpha / 255`.
pub fn pixel_from_color_alpha(c: Color, alpha: u8) -> u32 {
    let a = u32::from(alpha);
    let r = u32::from(c.r) * a / 255;
    let g = u32::from(c.g) * a / 255;
    let b = u32::from(c.b) * a / 255;
    (a << 24) | (r << 16) | (g << 8) | b
}

/// Sets alpha to 0xFF for pixels in a region that differ from the
/// background, fixing GDI's zero-alpha text output.
fn fix_alpha_region(ctx: &mut DrawCtx, rx: i32, ry: i32, rw: i32, rh: i32) {
    let x0 = rx.max(0);
    let y0 = ry.max(0);
    let x1 = (rx + rw).min(ctx.w);
    let y1 = (ry + rh).min(ctx.h);

    for py in y0..y1 {
        for px in x0..x1 {
            let idx = (py * ctx.w + px) as usize;
            if idx < ctx.buf.len() && ctx.buf[idx] != ctx.bg_pixel {
                ctx.buf[idx] |= 0xFF00_0000;
            }
        }
    }
}
