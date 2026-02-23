//! Workspaces widget — grouped container with inner active highlight.
//!
//! All workspace numbers sit inside a single container pill. The
//! active workspace gets a smaller inner accent pill; inactive
//! numbers are plain text on the container background.

use mosaico_core::config::bar::BarConfig;

use super::BarState;
use crate::bar::text::{DrawCtx, draw_pill, draw_text, measure_text};

/// Draws the workspace container and returns the X after it.
pub fn draw(ctx: &mut DrawCtx, start_x: i32, config: &BarConfig, state: &BarState) -> i32 {
    let labels: Vec<String> = (0..state.workspace_count)
        .map(|i| format!("{}", i + 1))
        .collect();
    let widths: Vec<i32> = labels.iter().map(|l| measure_text(ctx.dc, l)).collect();

    // Each cell is pill_padding * 2 + text width, spaced by workspace_gap.
    let cell_widths: Vec<i32> = widths
        .iter()
        .map(|tw| tw + config.pill_padding * 2)
        .collect();
    let total_cells: i32 = cell_widths.iter().sum();
    let gaps = (labels.len().saturating_sub(1) as i32) * config.workspace_gap;

    // Thin outer margin so the inner active pill sits snugly inside.
    let outer_pad = config.pill_padding / 3;
    let container_w = outer_pad + total_cells + gaps + outer_pad;

    let outer_y = pill_top(ctx.h);
    let outer_h = ctx.h - outer_y * 2;

    // Draw outer container pill.
    draw_pill(
        ctx,
        start_x,
        outer_y,
        container_w,
        outer_h,
        &config.colors.widget_background,
        config.pill_radius,
        &config.colors.pill_border,
        config.pill_border_width,
    );

    // Inner pill inset from the container edges.
    let inner_inset = outer_pad;
    let inner_y = outer_y + inner_inset;
    let inner_h = outer_h - inner_inset * 2;
    let inner_radius = (config.pill_radius - 1).max(2);

    // Draw each workspace number, with an accent pill behind the active one.
    let mut x = start_x + outer_pad;
    for (i, (label, &tw)) in labels.iter().zip(widths.iter()).enumerate() {
        let cell_w = tw + config.pill_padding * 2;
        let is_active = i == state.active_workspace;

        if is_active {
            draw_pill(
                ctx,
                x,
                inner_y,
                cell_w,
                inner_h,
                &config.colors.active_workspace,
                inner_radius,
                "",
                0,
            );
            draw_text(
                ctx,
                x + config.pill_padding,
                label,
                &config.colors.active_workspace_text,
            );
        } else {
            draw_text(
                ctx,
                x + config.pill_padding,
                label,
                &config.colors.inactive_workspace,
            );
        }

        x += cell_w + config.workspace_gap;
    }

    start_x + container_w + config.item_gap
}

/// Vertical offset for pill top edge (1px inset from bar edges).
fn pill_top(bar_height: i32) -> i32 {
    (bar_height / 8).max(1)
}
