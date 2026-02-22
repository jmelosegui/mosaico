//! Widget rendering dispatch for the status bar.
//!
//! Each widget lives in its own module and knows how to produce its
//! text content. The `left`/`right` arrays in `bar.toml` control
//! which widgets appear and in what order — removing an entry hides
//! that widget entirely.

pub mod clock;
pub mod cpu;
pub mod date;
pub mod layout;
pub mod ram;
mod system;
pub mod update;
pub mod workspaces;

use mosaico_core::config::bar::{BarConfig, WidgetConfig};

use super::text::{DrawCtx, draw_pill, draw_text, measure_text};

/// Snapshot of tiling and system state needed by bar widgets.
///
/// The daemon populates this and passes it to `Bar::update()`.
#[derive(Debug, Clone)]
pub struct BarState {
    pub active_workspace: usize,
    pub workspace_count: usize,
    pub layout_name: String,
    pub monocle: bool,
    pub cpu_usage: u32,
    /// Set by the daemon when a newer version is available.
    pub update_text: String,
}

impl Default for BarState {
    fn default() -> Self {
        Self {
            active_workspace: 0,
            workspace_count: 8,
            layout_name: "BSP".into(),
            monocle: false,
            cpu_usage: 0,
            update_text: String::new(),
        }
    }
}

/// Draws all left-side widgets. Returns the final cursor X.
pub fn draw_left(ctx: &mut DrawCtx, config: &BarConfig, state: &BarState) -> i32 {
    let mut x = config.padding;
    let mut drawn = 0;
    for widget in &config.left {
        if should_skip(widget, state) {
            continue;
        }
        if drawn > 0 {
            x = draw_separator(ctx, x, config);
        }
        x = match widget {
            WidgetConfig::Workspaces { .. } => workspaces::draw(ctx, x, config, state),
            _ => draw_pill_widget(ctx, x, config, state, widget),
        };
        drawn += 1;
    }
    x
}

/// Draws all right-side widgets. Returns the final right-edge X.
pub fn draw_right(ctx: &mut DrawCtx, config: &BarConfig, state: &BarState) -> i32 {
    let mut rx = ctx.w - config.padding;
    let mut drawn = 0;
    for widget in &config.right {
        if should_skip(widget, state) {
            continue;
        }
        if drawn > 0 {
            rx = draw_separator_right(ctx, rx, config);
        }
        rx = draw_pill_right(ctx, rx, config, state, widget);
        drawn += 1;
    }
    rx
}

/// Returns true if a widget should be skipped during rendering.
fn should_skip(widget: &WidgetConfig, state: &BarState) -> bool {
    if !widget.enabled() {
        return true;
    }
    // Hide the update widget when there is no update available.
    matches!(widget, WidgetConfig::Update { .. }) && state.update_text.is_empty()
}

// -- shared pill rendering ------------------------------------------------

/// Draws a widget as a pill (icon + text) and returns X after it.
fn draw_pill_widget(
    ctx: &mut DrawCtx,
    x: i32,
    config: &BarConfig,
    state: &BarState,
    widget: &WidgetConfig,
) -> i32 {
    let label = widget_label(state, widget);
    let fg = widget_fg(config, widget);
    let bg = &config.colors.widget_background;
    let tw = measure_text(ctx.dc, &label);
    let pill_w = tw + config.pill_padding * 2;
    let pill_y = pill_top(ctx.h);
    let pill_h = ctx.h - pill_y * 2;

    let border = &config.colors.pill_border;
    draw_pill(
        ctx,
        x,
        pill_y,
        pill_w,
        pill_h,
        bg,
        config.pill_radius,
        border,
    );
    draw_text(ctx, x + config.pill_padding, &label, &fg);
    x + pill_w + config.item_gap
}

/// Draws a widget right-aligned and returns the new right edge.
fn draw_pill_right(
    ctx: &mut DrawCtx,
    rx: i32,
    config: &BarConfig,
    state: &BarState,
    widget: &WidgetConfig,
) -> i32 {
    let label = widget_label(state, widget);
    let fg = widget_fg(config, widget);
    let bg = &config.colors.widget_background;
    let tw = measure_text(ctx.dc, &label);
    let pill_w = tw + config.pill_padding * 2;
    let pill_y = pill_top(ctx.h);
    let pill_h = ctx.h - pill_y * 2;
    let x = rx - pill_w;

    let border = &config.colors.pill_border;
    draw_pill(
        ctx,
        x,
        pill_y,
        pill_w,
        pill_h,
        bg,
        config.pill_radius,
        border,
    );
    draw_text(ctx, x + config.pill_padding, &label, &fg);
    x - config.item_gap
}

fn draw_separator(ctx: &mut DrawCtx, x: i32, config: &BarConfig) -> i32 {
    if config.separator.is_empty() {
        return x;
    }
    let padded = format!(" {} ", config.separator);
    draw_text(ctx, x, &padded, &config.colors.separator) + config.item_gap
}

fn draw_separator_right(ctx: &mut DrawCtx, rx: i32, config: &BarConfig) -> i32 {
    if config.separator.is_empty() {
        return rx;
    }
    let padded = format!(" {} ", config.separator);
    let tw = measure_text(ctx.dc, &padded);
    let x = rx - tw;
    draw_text(ctx, x, &padded, &config.colors.separator);
    x - config.item_gap
}

// -- helpers --------------------------------------------------------------

/// Combines icon + text into the label string shown inside a pill.
fn widget_label(state: &BarState, widget: &WidgetConfig) -> String {
    let icon = widget.icon();
    let text = widget_text(state, widget);
    if icon.is_empty() {
        text
    } else if text.is_empty() {
        icon.to_string()
    } else {
        format!("{icon} {text}")
    }
}

/// Returns the raw text content for a widget (no icon).
fn widget_text(state: &BarState, widget: &WidgetConfig) -> String {
    match widget {
        WidgetConfig::Workspaces { .. } => String::new(),
        WidgetConfig::Layout { .. } => layout::text(state),
        WidgetConfig::Clock { format, .. } => clock::text(format),
        WidgetConfig::Date { format, .. } => date::text(format),
        WidgetConfig::Ram { .. } => ram::text(),
        WidgetConfig::Cpu { .. } => cpu::text(state.cpu_usage),
        WidgetConfig::Update { .. } => update::text(state),
    }
}

/// Returns the foreground color hex for a widget.
fn widget_fg(config: &BarConfig, widget: &WidgetConfig) -> String {
    match widget {
        WidgetConfig::Update { .. } => config.colors.accent.clone(),
        _ => config.colors.foreground.clone(),
    }
}

/// Vertical offset for pill top edge.
fn pill_top(bar_height: i32) -> i32 {
    (bar_height / 8).max(1)
}
