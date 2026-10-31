//! The window's own title bar and resize edges, since it draws its
//! frame in the theme instead of the system's.

use eframe::egui::{
    Align, Color32, Context, CursorIcon, Layout, Rect, ResizeDirection, RichText, Sense, Ui,
    ViewportCommand, pos2, vec2,
};

use crate::icons::{self, Icon};
use crate::theme::{self, Colors};

/// How far from the window's edge a drag resizes it, in points.
const EDGE: f32 = 5.0;

/// The logo, the title and the minimize, maximize and close buttons.
/// Dragging the bar moves the window; double-clicking it maximizes it.
pub fn title_bar(ui: &mut Ui, c: &Colors) {
    let bar = ui.max_rect();
    let drag = ui.interact(
        bar,
        ui.scope_id().with("title drag"),
        Sense::click_and_drag(),
    );
    let maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
    if drag.drag_started() {
        ui.send_viewport_cmd(ViewportCommand::StartDrag);
    }
    if drag.double_clicked() {
        ui.send_viewport_cmd(ViewportCommand::Maximized(!maximized));
    }

    ui.allocate_ui_with_layout(bar.size(), Layout::left_to_right(Align::Center), |ui| {
        ui.add_space(12.0);
        let (logo, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
        paint_logo(ui, logo);
        ui.add_space(4.0);
        ui.label(
            RichText::new("mosaico settings")
                .font(theme::regular(12.0))
                .color(c.dim_text),
        );

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            if caption(ui, c, Icon::Close, "Close", true).clicked() {
                ui.send_viewport_cmd(ViewportCommand::Close);
            }
            let (icon, tip) = if maximized {
                (Icon::Restore, "Restore")
            } else {
                (Icon::Maximize, "Maximize")
            };
            if caption(ui, c, icon, tip, false).clicked() {
                ui.send_viewport_cmd(ViewportCommand::Maximized(!maximized));
            }
            if caption(ui, c, Icon::Minimize, "Minimize", false).clicked() {
                ui.send_viewport_cmd(ViewportCommand::Minimized(true));
            }
        });
    });
}

fn caption(
    ui: &mut Ui,
    c: &Colors,
    icon: Icon,
    label: &str,
    close: bool,
) -> eframe::egui::Response {
    let (rect, response) = ui.allocate_exact_size(vec2(46.0, 36.0), Sense::click());
    let response = response.on_hover_text(label);
    response
        .widget_info(|| eframe::egui::WidgetInfo::labeled(eframe::egui::Role::Button, true, label));
    let (fill, color) = match (response.hovered(), close) {
        (true, true) => (c.bad, c.on_accent),
        (true, false) => (c.hover, c.text),
        (false, _) => (Color32::TRANSPARENT, c.dim_text),
    };
    ui.painter().rect_filled(rect, 0.0, fill);
    let glyph = Rect::from_center_size(rect.center(), vec2(14.0, 14.0));
    icons::paint(ui.painter(), glyph, icon, color, 1.25);
    response
}

/// Four tiles in the colors of the mosaico icon.
fn paint_logo(ui: &Ui, rect: Rect) {
    let painter = ui.painter();
    painter.rect_filled(rect, 3.0, Color32::from_rgb(0x1e, 0x1e, 0x2e));
    let r = rect.shrink(2.5);
    let (w, h, gap) = (r.width(), r.height(), 1.5);
    let half_w = (w - gap) / 2.0;
    let half_h = (h - gap) / 2.0;
    let right = r.left() + half_w + gap;
    let bottom = r.top() + half_h + gap;
    let quarter = (half_w - gap) / 2.0;
    let tiles = [
        (Rect::from_min_size(r.min, vec2(half_w, h)), 0x89b4fa),
        (
            Rect::from_min_size(pos2(right, r.top()), vec2(half_w, half_h)),
            0xf5c2e7,
        ),
        (
            Rect::from_min_size(pos2(right, bottom), vec2(quarter, half_h)),
            0xa6e3a1,
        ),
        (
            Rect::from_min_size(pos2(right + quarter + gap, bottom), vec2(quarter, half_h)),
            0xf9e2af,
        ),
    ];
    for (tile, rgb) in tiles {
        let [_, red, green, blue] = u32::to_be_bytes(rgb);
        painter.rect_filled(tile, 1.0, Color32::from_rgb(red, green, blue));
    }
}

/// Lets the window be resized from its edges and corners, unless it is
/// maximized.
pub fn resize_edges(ctx: &Context) {
    if ctx.input(|i| i.viewport().maximized.unwrap_or(false)) {
        return;
    }
    let screen = ctx.content_rect();
    let (l, r, t, b) = (screen.left(), screen.right(), screen.top(), screen.bottom());
    let edges = [
        (
            Rect::from_min_max(pos2(l, t), pos2(l + EDGE, t + EDGE)),
            ResizeDirection::NorthWest,
        ),
        (
            Rect::from_min_max(pos2(r - EDGE, t), pos2(r, t + EDGE)),
            ResizeDirection::NorthEast,
        ),
        (
            Rect::from_min_max(pos2(l, b - EDGE), pos2(l + EDGE, b)),
            ResizeDirection::SouthWest,
        ),
        (
            Rect::from_min_max(pos2(r - EDGE, b - EDGE), pos2(r, b)),
            ResizeDirection::SouthEast,
        ),
        (
            Rect::from_min_max(pos2(l, t), pos2(r, t + EDGE)),
            ResizeDirection::North,
        ),
        (
            Rect::from_min_max(pos2(l, b - EDGE), pos2(r, b)),
            ResizeDirection::South,
        ),
        (
            Rect::from_min_max(pos2(l, t), pos2(l + EDGE, b)),
            ResizeDirection::West,
        ),
        (
            Rect::from_min_max(pos2(r - EDGE, t), pos2(r, b)),
            ResizeDirection::East,
        ),
    ];
    let pointer = ctx.pointer_hover_pos();
    let pressed = ctx.input(|i| i.pointer.primary_pressed());
    let Some((_, direction)) = pointer.and_then(|p| edges.into_iter().find(|(e, _)| e.contains(p)))
    else {
        return;
    };
    ctx.set_cursor_icon(cursor(direction));
    if pressed {
        ctx.send_viewport_cmd(ViewportCommand::BeginResize(direction));
    }
}

fn cursor(direction: ResizeDirection) -> CursorIcon {
    match direction {
        ResizeDirection::North => CursorIcon::ResizeNorth,
        ResizeDirection::South => CursorIcon::ResizeSouth,
        ResizeDirection::East => CursorIcon::ResizeEast,
        ResizeDirection::West => CursorIcon::ResizeWest,
        ResizeDirection::NorthEast => CursorIcon::ResizeNorthEast,
        ResizeDirection::SouthEast => CursorIcon::ResizeSouthEast,
        ResizeDirection::NorthWest => CursorIcon::ResizeNorthWest,
        ResizeDirection::SouthWest => CursorIcon::ResizeSouthWest,
    }
}
