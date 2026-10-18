//! DIB-based rendering orchestrator for the status bar.
//!
//! Paints on a [`Canvas`]: fills the background, sets the font and
//! delegates widget drawing, then applies the pixels via
//! `UpdateLayeredWindow`.

use mosaico_core::bar::{BarContent, ItemRegion};
use mosaico_core::config::bar::BarConfig;
use mosaico_core::{Rect, Result};
use windows::Win32::Foundation::{COLORREF, HWND, POINT, SIZE};
use windows::Win32::Graphics::Gdi::{
    BLENDFUNCTION, CreateFontW, DeleteObject, FONT_CHARSET, FONT_CLIP_PRECISION,
    FONT_OUTPUT_PRECISION, FONT_QUALITY, SelectObject, SetBkMode, TRANSPARENT,
};
use windows::Win32::UI::WindowsAndMessaging::{ULW_ALPHA, UpdateLayeredWindow};
use windows::core::PCWSTR;

use super::canvas::Canvas;
use super::pixels::leave_no_hole;
use super::text::{DrawCtx, pixel_from_color_alpha};
use super::widgets;
use crate::border::Color;
use crate::error::OrPlatform;

/// Renders the full bar content, applies it to the layered window over
/// `area` of the screen, and returns where each item went. A bar that
/// `takes_input` leaves no pixel of a clickable item fully transparent, so
/// the whole item takes the pointer.
///
/// A bar with no area has nothing to draw, which is not an error.
pub fn render_bar(
    hwnd: HWND,
    area: Rect,
    config: &BarConfig,
    content: &BarContent,
    takes_input: bool,
) -> Result<Vec<ItemRegion>> {
    if area.width <= 0 || area.height <= 0 {
        return Ok(Vec::new());
    }
    let mut canvas = Canvas::new(area.width, area.height)?;
    let regions = paint(&mut canvas, config, content, takes_input);
    present(hwnd, &canvas, area).or_platform("show the bar's pixels")?;
    Ok(regions)
}

/// Fills `canvas` with the background, draws the items of `content` on
/// it in the configured font, and returns where each item went. When the
/// bar `takes_input`, clickable items keep no fully transparent pixel.
pub(super) fn paint(
    canvas: &mut Canvas,
    config: &BarConfig,
    content: &BarContent,
    takes_input: bool,
) -> Vec<ItemRegion> {
    // Fill background (with opacity for transparency support)
    let bg = Color::from_hex(&config.colors.background).unwrap_or(Color {
        r: 0x1e,
        g: 0x1e,
        b: 0x2e,
    });
    let alpha = (config.background_opacity * 255 / 100) as u8;
    let bg_pixel = pixel_from_color_alpha(bg, alpha);
    canvas.pixels().fill(bg_pixel);

    // Create font from config
    let font_wide: Vec<u16> = config
        .font
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let weight = if config.font_bold { 700 } else { 400 };
    let (dc, w, h) = (canvas.dc, canvas.w, canvas.h);
    // SAFETY: the font is made for this paint and selected into the
    // canvas's memory DC, which the widgets draw on; it is put back and
    // deleted before return.
    unsafe {
        let font = CreateFontW(
            config.font_size,
            0,
            0,
            0,
            weight,
            config.font_italic as u32,
            config.font_underline as u32,
            0,
            FONT_CHARSET(0),
            FONT_OUTPUT_PRECISION(0),
            FONT_CLIP_PRECISION(0),
            FONT_QUALITY(0),
            0,
            PCWSTR(font_wide.as_ptr()),
        );
        let old_font = SelectObject(dc, font.into());
        let _ = SetBkMode(dc, TRANSPARENT);

        let mut ctx = DrawCtx {
            dc,
            buf: canvas.pixels(),
            w,
            h,
            bg_pixel,
        };
        // Delegate to widget renderers
        let regions = widgets::draw(&mut ctx, config, content);

        SelectObject(dc, old_font);
        let _ = DeleteObject(font.into());
        if takes_input {
            leave_no_hole(canvas.pixels(), w, &regions);
        }
        regions
    }
}

/// Calls `UpdateLayeredWindow` to atomically present the canvas at
/// `area` of the screen.
fn present(hwnd: HWND, canvas: &Canvas, area: Rect) -> windows::core::Result<()> {
    let blend = BLENDFUNCTION {
        BlendOp: 0,
        SourceConstantAlpha: 255,
        AlphaFormat: 1,
        ..Default::default()
    };
    // SAFETY: UpdateLayeredWindow atomically sets the window's position,
    // size, and pixel content from the canvas's memory DC.
    unsafe {
        UpdateLayeredWindow(
            hwnd,
            Some(canvas.screen_dc),
            Some(&POINT {
                x: area.x,
                y: area.y,
            }),
            Some(&SIZE {
                cx: area.width,
                cy: area.height,
            }),
            Some(canvas.dc),
            Some(&POINT::default()),
            COLORREF(0),
            Some(&blend),
            ULW_ALPHA,
        )
    }
}

#[cfg(test)]
#[path = "render_tests.rs"]
mod tests;
