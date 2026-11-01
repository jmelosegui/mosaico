//! The row of one widget: its grip, name, switch and options.

use eframe::egui::{
    Align, CursorIcon, Frame, Key, Layout, Margin, Modifiers, Rect, RichText, Role, Sense, Stroke,
    Ui, UiBuilder, WidgetInfo, pos2, vec2,
};
use mosaico_core::config::{BarWidget, BarZone};

use super::{Change, Ctx, ROW, grip_id, icon, label, note};
use crate::icons::{self, Icon};
use crate::theme::{self, Colors};
use crate::widgets::{self, ColorPicker};

/// Where a widget's name starts: after the grip and the icon.
const TEXT: f32 = 51.0;

pub(super) fn row(
    ui: &mut Ui,
    cx: &mut Ctx,
    zone: BarZone,
    index: usize,
    widget: &mut BarWidget,
    last: bool,
) {
    let c = cx.c;
    let name = label(widget.kind);
    let place = (zone, index);
    let selected = cx.state.open == Some(place);
    let dragged = cx.state.drag == Some(place);
    let unsaved = cx.saved.zone(zone).get(index) != Some(&*widget);
    let (rect, body) = ui.allocate_exact_size(vec2(ui.available_width(), ROW), Sense::click());
    body.widget_info(|| {
        WidgetInfo::selected(Role::Button, true, selected, format!("{name} options"))
    });
    if body.clicked() {
        cx.state.open = if selected { None } else { Some(place) };
    }
    cx.seen.rows.push((zone, index, rect));
    let painter = ui.painter();
    if selected || dragged {
        painter.rect_filled(rect, 0.0, c.selected);
    } else if body.hovered() {
        painter.rect_filled(rect, 0.0, c.hover);
    }
    if dragged {
        painter.rect_stroke(
            rect.shrink(1.0),
            6.0,
            Stroke::new(1.0, c.accent),
            eframe::egui::StrokeKind::Inside,
        );
    }
    let grip = grip(ui, cx, rect, (zone, index, last), name);
    if grip.drag_started() {
        cx.state.drag = Some(place);
    }
    if grip.drag_stopped() {
        cx.seen.released = true;
    }
    let painter = ui.painter();
    let color = if widget.enabled { c.text } else { c.faint_text };
    let tint = if widget.enabled {
        c.accent
    } else {
        c.faint_text
    };
    let glyph = Rect::from_center_size(rect.left_center() + vec2(35.5, 0.0), vec2(15.0, 15.0));
    icons::paint(painter, glyph, icon(widget.kind), tint, 1.5);
    let note = note(widget);
    let text_y = rect.center().y - if note.is_some() { 14.0 } else { 8.0 };
    painter.text(
        pos2(rect.left() + TEXT, text_y),
        eframe::egui::Align2::LEFT_TOP,
        name,
        if selected {
            theme::bold(13.0)
        } else {
            theme::regular(13.0)
        },
        color,
    );
    if let Some(note) = note {
        painter.text(
            pos2(rect.left() + TEXT, text_y + 17.0),
            eframe::egui::Align2::LEFT_TOP,
            note,
            theme::regular(11.0),
            c.faint_text,
        );
    }
    if unsaved {
        painter.circle_filled(pos2(rect.right() - 62.0, rect.center().y), 3.0, c.accent);
    }
    let area = Rect::from_min_size(
        pos2(rect.right() - 46.0, rect.center().y - 10.0),
        vec2(34.0, 20.0),
    );
    let mut toggle = ui.new_child(UiBuilder::new().max_rect(area));
    widgets::toggle(&mut toggle, c, name, &mut widget.enabled);
    if selected {
        options(ui, cx, widget);
    } else if !last {
        let y = rect.bottom();
        ui.painter().line_segment(
            [pos2(rect.left(), y), pos2(rect.right(), y)],
            Stroke::new(1.0, c.line),
        );
    }
}

/// The grip of a row: dragged with the pointer, or moved with the
/// arrow keys while it has the focus.
fn grip(
    ui: &mut Ui,
    cx: &mut Ctx,
    row: Rect,
    (zone, index, last): (BarZone, usize, bool),
    name: &str,
) -> eframe::egui::Response {
    let area = Rect::from_center_size(row.left_center() + vec2(13.0, 0.0), vec2(20.0, 32.0));
    let id = grip_id(ui, zone, index);
    let response = ui.interact(area, id, Sense::click_and_drag());
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, format!("Move {name}")));
    if cx.state.focus == Some(id) {
        response.request_focus();
        cx.state.focus = None;
    }
    let color = if response.hovered() || response.dragged() {
        cx.c.text
    } else {
        cx.c.faint_text
    };
    // Drawn larger than the box, so the dots stand apart as in the sketch.
    let glyph = Rect::from_center_size(area.center(), vec2(19.0, 19.0));
    icons::paint(ui.painter(), glyph, Icon::Grip, color, 1.4);
    if response.hovered() || response.dragged() {
        ui.ctx().set_cursor_icon(if response.dragged() {
            CursorIcon::Grabbing
        } else {
            CursorIcon::Grab
        });
    }
    if response.has_focus() {
        keys(ui, cx, zone, index, last);
    }
    response
}

fn keys(ui: &Ui, cx: &mut Ctx, zone: BarZone, index: usize, last: bool) {
    let pressed = |key| ui.input_mut(|i| i.consume_key(Modifiers::NONE, key));
    let to = if pressed(Key::ArrowUp) && index > 0 {
        (zone, index - 1)
    } else if pressed(Key::ArrowDown) && !last {
        (zone, index + 1)
    } else if pressed(Key::ArrowLeft) && zone == BarZone::Right {
        (BarZone::Left, usize::MAX)
    } else if pressed(Key::ArrowRight) && zone == BarZone::Left {
        (BarZone::Right, usize::MAX)
    } else {
        return;
    };
    cx.change = Some(Change::Move {
        from: (zone, index),
        to,
        by_key: true,
    });
}

/// The format, icon and color of an open widget.
fn options(ui: &mut Ui, cx: &mut Ctx, widget: &mut BarWidget) {
    let c = cx.c;
    Frame::new()
        .fill(c.selected)
        .inner_margin(Margin {
            left: 42,
            right: 14,
            top: 6,
            bottom: 12,
        })
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 8.0;
            if let Some(format) = &widget.format {
                let mut text = format.clone();
                option(ui, c, "Format", |ui| {
                    widgets::text(ui, c, "Format", 178.0, &mut text, "", false)
                });
                widget.format = Some(text);
            }
            option(ui, c, "Icon", |ui| {
                let choices = super::glyphs::choices(&widget.icon);
                widgets::icon_dropdown(ui, c, "Icon", 190.0, &choices, &mut widget.icon, false)
            });
            let fallback = cx
                .theme
                .default_color("colors.foreground")
                .unwrap_or_default();
            let picker = ColorPicker {
                label: "Color",
                theme: cx.theme,
                fallback: &fallback,
                allow_none: false,
                changed: false,
            };
            option(ui, c, "Color", |ui| {
                widgets::color(ui, c, &picker, &mut widget.color)
            });
        });
    let y = ui.cursor().top();
    let rect = ui.max_rect();
    ui.painter().line_segment(
        [pos2(rect.left(), y), pos2(rect.right(), y)],
        Stroke::new(1.0, c.line),
    );
}

/// A label with its control at the far end.
fn option<R>(ui: &mut Ui, c: &Colors, name: &str, control: impl FnOnce(&mut Ui) -> R) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(name)
                .font(theme::regular(12.0))
                .color(c.dim_text),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            control(ui);
        });
    });
}
