//! Draws a bar's items into its pixel buffer.
//!
//! Core decides what each bar shows, as a list of
//! [`BarItem`](mosaico_core::bar::BarItem)s per side. This module lays
//! them out from the bar's edges and draws them with GDI.

pub mod active_window;
pub mod workspaces;

use mosaico_core::bar::BarItem;
use mosaico_core::config::bar::BarConfig;

use super::text::{DrawCtx, draw_pill, draw_text, measure_text};

/// Draws the left items from the left edge. Returns the final cursor X.
pub fn draw_left(ctx: &mut DrawCtx, config: &BarConfig, items: &[BarItem]) -> i32 {
    let mut x = config.padding;
    for item in items {
        x = match item {
            BarItem::Workspaces { labels, active } => {
                workspaces::draw(ctx, x, config, labels, *active)
            }
            BarItem::WindowIcon(window) => active_window::draw(ctx, x, config, *window),
            BarItem::Pill {
                label,
                text_color,
                border_color,
            } => {
                let pill_w = pill_width(ctx, config, label);
                draw_label_pill(ctx, x, pill_w, config, label, text_color, border_color);
                x + pill_w + config.item_gap
            }
            BarItem::Separator(text) => {
                draw_text(ctx, x, text, &config.colors.separator) + config.item_gap
            }
        };
    }
    x
}

/// Draws the right items from the right edge. Returns the final
/// right-edge X.
///
/// The right side only holds pills and separators (see
/// [`BarContent::right`](mosaico_core::bar::BarContent::right)), so the
/// workspace container and the window icon are not drawn here.
pub fn draw_right(ctx: &mut DrawCtx, config: &BarConfig, items: &[BarItem]) -> i32 {
    let mut rx = ctx.w - config.padding;
    for item in items {
        rx = match item {
            BarItem::Pill {
                label,
                text_color,
                border_color,
            } => {
                let pill_w = pill_width(ctx, config, label);
                let x = rx - pill_w;
                draw_label_pill(ctx, x, pill_w, config, label, text_color, border_color);
                x - config.item_gap
            }
            BarItem::Separator(text) => {
                let x = rx - measure_text(ctx.dc, text);
                draw_text(ctx, x, text, &config.colors.separator);
                x - config.item_gap
            }
            // Never on the right: core makes those widgets pills there.
            BarItem::Workspaces { .. } | BarItem::WindowIcon(_) => rx,
        };
    }
    rx
}

/// Width of the pill around `label`.
fn pill_width(ctx: &DrawCtx, config: &BarConfig, label: &str) -> i32 {
    measure_text(ctx.dc, label) + config.pill_padding * 2
}

/// Draws a pill `width` wide at `x`, with `label` inside it.
fn draw_label_pill(
    ctx: &mut DrawCtx,
    x: i32,
    width: i32,
    config: &BarConfig,
    label: &str,
    text_color: &str,
    border_color: &str,
) {
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
        border_color,
        config.pill_border_width,
    );
    draw_text(ctx, x + config.pill_padding, label, text_color);
}

/// Vertical offset for pill top edge.
fn pill_top(bar_height: i32) -> i32 {
    (bar_height / 8).max(1)
}
