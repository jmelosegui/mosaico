//! Draws a widget's pill: its outline, its icon and its label.

use mosaico_core::bar::BarIcon;
use mosaico_core::config::bar::BarConfig;
use windows::Win32::Graphics::Gdi::{GetTextMetricsW, HDC, TEXTMETRICW};

use super::super::text::{DrawCtx, draw_pill, draw_text, measure_text};
use super::icon::{self, IconSize};

/// What one pill shows.
pub struct Pill<'a> {
    pub icon: Option<BarIcon>,
    pub label: &'a str,
    pub text_color: &'a str,
    pub border_color: &'a str,
}

/// Width of `pill`.
pub fn width(ctx: &DrawCtx, config: &BarConfig, pill: &Pill) -> i32 {
    measure_text(ctx.dc, pill.label) + config.pill_padding * 2 + icon_room(ctx, pill)
}

/// The room `pill`'s icon takes before its label, with the gap after it
/// when there is a label.
fn icon_room(ctx: &DrawCtx, pill: &Pill) -> i32 {
    if pill.icon.is_none() {
        return 0;
    }
    let size = IconSize::for_em(em_height(ctx.dc));
    if pill.label.is_empty() {
        size.size
    } else {
        size.size + size.gap
    }
}

/// Draws `pill` `width` wide at `x`.
pub fn draw(ctx: &mut DrawCtx, x: i32, width: i32, config: &BarConfig, pill: &Pill) {
    let pill_y = pill_top(ctx.h);
    let pill_h = ctx.h - pill_y * 2;

    draw_pill(
        ctx,
        x,
        pill_y,
        width,
        pill_h,
        &config.colors.widget_background,
        config.pill_radius,
        pill.border_color,
        config.pill_border_width,
    );
    let x = x + config.pill_padding;
    if let Some(drawing) = pill.icon {
        let size = IconSize::for_em(em_height(ctx.dc));
        icon::draw(ctx, x, drawing, &size, pill.text_color);
    }
    draw_text(ctx, x + icon_room(ctx, pill), pill.label, pill.text_color);
}

/// Vertical offset for pill top edge.
fn pill_top(bar_height: i32) -> i32 {
    (bar_height / 8).max(1)
}

/// The em of the DC's font in pixels: its height without the room it
/// leaves above for accents.
fn em_height(dc: HDC) -> i32 {
    let mut metrics = TEXTMETRICW::default();
    // SAFETY: GetTextMetricsW fills the metrics of the font selected into
    // the DC, which is valid for the duration of the call.
    unsafe {
        let _ = GetTextMetricsW(dc, &mut metrics);
    }
    metrics.tmHeight - metrics.tmInternalLeading
}
