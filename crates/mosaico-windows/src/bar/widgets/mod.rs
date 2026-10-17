//! Draws a bar's items into its pixel buffer, and says where they went.
//!
//! Core decides what each bar shows, as a list of
//! [`BarItem`](mosaico_core::bar::BarItem)s per side. This module lays
//! them out from the bar's edges, draws them with GDI and returns the
//! region of each, which the content says what to make of.

pub mod active_window;
pub mod workspaces;

use mosaico_core::Rect;
use mosaico_core::bar::{BarContent, BarItem, BarSide, ItemRegion};
use mosaico_core::config::bar::BarConfig;

use super::text::{DrawCtx, draw_pill, draw_text, measure_text};

/// Draws every item of `content`, and returns where each went, in the
/// order they were drawn: the left side, then the right.
pub fn draw(ctx: &mut DrawCtx, config: &BarConfig, content: &BarContent) -> Vec<ItemRegion> {
    let mut regions = Vec::new();
    draw_left(ctx, config, content, &mut regions);
    draw_right(ctx, config, content, &mut regions);
    regions
}

/// Draws the left items from the left edge, adding where they went to
/// `regions`: each item, then the labels of the workspaces item.
fn draw_left(
    ctx: &mut DrawCtx,
    config: &BarConfig,
    content: &BarContent,
    regions: &mut Vec<ItemRegion>,
) {
    let mut x = config.padding;
    for (index, item) in content.left.iter().enumerate() {
        let start = x;
        let mut cells = Vec::new();
        x = match item {
            BarItem::Workspaces { labels, active } => {
                workspaces::draw(ctx, x, config, labels, *active, &mut cells)
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
        let rect = taken(start, x, config.item_gap, ctx.h);
        regions.extend(content.region(BarSide::Left, index, rect));
        let labels = cells.into_iter().enumerate();
        regions.extend(labels.map(|(label, cell)| content.label_region(label, cell)));
    }
}

/// Draws the right items from the right edge, adding where they went to
/// `regions`.
///
/// The right side only holds pills and separators (see
/// [`BarContent::right`](mosaico_core::bar::BarContent::right)), so the
/// workspace container and the window icon are not drawn here.
fn draw_right(
    ctx: &mut DrawCtx,
    config: &BarConfig,
    content: &BarContent,
    regions: &mut Vec<ItemRegion>,
) {
    let mut rx = ctx.w - config.padding;
    for (index, item) in content.right.iter().enumerate() {
        let (x, next) = match item {
            BarItem::Pill {
                label,
                text_color,
                border_color,
            } => {
                let pill_w = pill_width(ctx, config, label);
                let x = rx - pill_w;
                draw_label_pill(ctx, x, pill_w, config, label, text_color, border_color);
                (x, x - config.item_gap)
            }
            BarItem::Separator(text) => {
                let x = rx - measure_text(ctx.dc, text);
                draw_text(ctx, x, text, &config.colors.separator);
                (x, x - config.item_gap)
            }
            // Never on the right: core makes those widgets pills there.
            BarItem::Workspaces { .. } | BarItem::WindowIcon(_) => (rx, rx),
        };
        let rect = Rect::new(x, 0, rx - x, ctx.h);
        regions.extend(content.region(BarSide::Right, index, rect));
        rx = next;
    }
}

/// What an item drawn from `start` took on a bar `height` tall, given
/// `next`, where the item after it starts: all of it but the `gap` that
/// follows, and nothing when the item drew nothing, as the window icon
/// without a window. Its full height, so the top of the screen finds it.
fn taken(start: i32, next: i32, gap: i32, height: i32) -> Rect {
    let width = if next > start { next - start - gap } else { 0 };
    Rect::new(start, 0, width.max(0), height)
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
