//! Draws a bar's items into its pixel buffer, and says where they went.
//!
//! Core decides what each bar shows, as a list of
//! [`BarItem`](mosaico_core::bar::BarItem)s per side. This module lays
//! them out from the bar's edges, draws them with GDI and returns the
//! region of each, which the content says what to make of.

pub mod active_window;
mod icon;
mod pill;
pub mod workspaces;

use mosaico_core::Rect;
use mosaico_core::bar::{BarContent, BarItem, BarSide, ItemRegion};
use mosaico_core::config::bar::BarConfig;

use self::pill::Pill;
use super::text::{DrawCtx, draw_text, measure_text};

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
                icon,
                label,
                text_color,
                border_color,
            } => {
                let pill = Pill {
                    icon: *icon,
                    label,
                    text_color,
                    border_color,
                };
                let pill_w = pill::width(ctx, config, &pill);
                pill::draw(ctx, x, pill_w, config, &pill);
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
                icon,
                label,
                text_color,
                border_color,
            } => {
                let pill = Pill {
                    icon: *icon,
                    label,
                    text_color,
                    border_color,
                };
                let pill_w = pill::width(ctx, config, &pill);
                let x = rx - pill_w;
                pill::draw(ctx, x, pill_w, config, &pill);
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
