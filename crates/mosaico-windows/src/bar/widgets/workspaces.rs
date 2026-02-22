//! Workspaces widget — renders one pill per workspace number.
//!
//! The active workspace gets a highlighted pill; inactive workspaces
//! get a dimmer background. This is the only widget with custom
//! multi-element rendering.

use mosaico_core::config::bar::BarConfig;

use super::BarState;
use crate::bar::text::{DrawCtx, draw_pill, draw_text, measure_text};

/// Draws workspace indicator pills and returns the X after the last one.
pub fn draw(ctx: &mut DrawCtx, start_x: i32, config: &BarConfig, state: &BarState) -> i32 {
    let mut x = start_x;
    let pill_y = pill_top(ctx.h);
    let pill_h = ctx.h - pill_y * 2;

    for i in 0..state.workspace_count {
        let label = format!("{}", i + 1);
        let is_active = i == state.active_workspace;

        let (fg, bg) = if is_active {
            (
                &config.colors.active_workspace_text,
                &config.colors.active_workspace,
            )
        } else {
            (
                &config.colors.inactive_workspace,
                &config.colors.widget_background,
            )
        };

        let tw = measure_text(ctx.dc, &label);
        let pill_w = tw + config.pill_padding * 2;

        draw_pill(
            ctx,
            x,
            pill_y,
            pill_w,
            pill_h,
            bg,
            config.pill_radius,
            &config.colors.pill_border,
        );
        draw_text(ctx, x + config.pill_padding, &label, fg);
        x += pill_w + config.workspace_gap;
    }

    x + config.item_gap
}

/// Vertical offset for pill top edge (1px inset from bar edges).
fn pill_top(bar_height: i32) -> i32 {
    (bar_height / 8).max(1)
}
