//! A number on a track, with the number in a pill beside it.

use eframe::egui::{
    Align, Color32, CursorIcon, Frame, Layout, Margin, Rect, Response, RichText, Role, Sense,
    Stroke, Ui, WidgetInfo, emath::Numeric, pos2, vec2,
};

use super::fields::drag_value;
use crate::theme::{self, Colors};

/// A track over `track`, then a pill with the number and its `unit`,
/// where any number in `range` can be typed. `label` names both for
/// assistive technology. Lays out right to left, as the controls of a row
/// do.
#[allow(clippy::too_many_arguments)]
pub fn slider<N: Numeric>(
    ui: &mut Ui,
    c: &Colors,
    label: &str,
    value: &mut N,
    range: std::ops::RangeInclusive<N>,
    track: std::ops::RangeInclusive<N>,
    speed: f64,
    unit: &str,
    changed: bool,
) -> Response {
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing.x = 12.0;
        let field = pill(ui, c, label, value, range, speed, unit, changed);
        let track = self::track(ui, c, label, value, &track);
        track | field
    })
    .inner
}

/// How wide the track of a [`slider`] is.
const SLIDER_WIDTH: f32 = 200.0;

#[allow(clippy::too_many_arguments)]
fn pill<N: Numeric>(
    ui: &mut Ui,
    c: &Colors,
    label: &str,
    value: &mut N,
    range: std::ops::RangeInclusive<N>,
    speed: f64,
    unit: &str,
    changed: bool,
) -> Response {
    let font = theme::regular(12.0);
    let stroke = if changed {
        Stroke::new(1.0, c.accent)
    } else {
        Stroke::NONE
    };
    let size = vec2(ui.available_width(), 24.0);
    let layout = Layout::right_to_left(Align::Center);
    ui.allocate_ui_with_layout(size, layout, |ui| {
        Frame::new()
            .fill(c.control)
            .stroke(stroke)
            .corner_radius(5)
            .inner_margin(Margin::symmetric(8, 3))
            .show(ui, |ui| {
                let spacing = ui.spacing_mut();
                spacing.item_spacing.x = 0.0;
                spacing.interact_size = vec2(0.0, 18.0);
                spacing.button_padding = vec2(0.0, 0.0);
                let w = &mut ui.visuals_mut().widgets;
                for state in [&mut w.inactive, &mut w.hovered, &mut w.active] {
                    state.weak_bg_fill = Color32::TRANSPARENT;
                    state.bg_fill = c.control;
                    state.bg_stroke = Stroke::NONE;
                }
                ui.style_mut().override_font_id = Some(font.clone());
                // Right to left: the unit goes in first so it ends up after
                // the number.
                if !unit.is_empty() {
                    ui.label(RichText::new(unit).font(font));
                    // A percent sign sits against its number.
                    ui.add_space(if unit == "%" {
                        -ui.spacing().item_spacing.x
                    } else {
                        4.0
                    });
                }
                let field = ui
                    .add(drag_value(value, range, speed))
                    .on_hover_text("Drag, or click to type");
                field.widget_info(|| WidgetInfo::labeled(Role::SpinButton, true, label));
                field
            })
            .inner
    })
    .inner
}

fn track<N: Numeric>(
    ui: &mut Ui,
    c: &Colors,
    label: &str,
    value: &mut N,
    range: &std::ops::RangeInclusive<N>,
) -> Response {
    let (lo, hi) = (range.start().to_f64(), range.end().to_f64());
    let (rect, mut response) =
        ui.allocate_exact_size(vec2(SLIDER_WIDTH, 26.0), Sense::click_and_drag());
    let knob_room = rect.shrink2(vec2(8.0, 0.0));
    if let Some(pointer) = response.interact_pointer_pos() {
        let t = ((pointer.x - knob_room.left()) / knob_room.width()).clamp(0.0, 1.0);
        let x = lo + f64::from(t) * (hi - lo);
        let x = if N::INTEGRAL {
            x.round()
        } else {
            (x * 100.0).round() / 100.0
        };
        let picked = N::from_f64(x);
        if picked != *value {
            *value = picked;
            response.mark_changed();
        }
    }
    let shown = value.to_f64();
    response.widget_info(|| WidgetInfo::slider(true, shown, label));
    if !ui.is_rect_visible(rect) {
        return response;
    }
    let t = if hi > lo {
        ((shown - lo) / (hi - lo)).clamp(0.0, 1.0) as f32
    } else {
        0.0
    };
    let y = rect.center().y;
    let x = knob_room.left() + knob_room.width() * t;
    let painter = ui.painter();
    let line = |from: f32, to: f32, color| {
        let bar = Rect::from_min_max(pos2(from, y - 2.0), pos2(to, y + 2.0));
        painter.rect_filled(bar, 2.0, color);
    };
    line(knob_room.left(), knob_room.right(), c.control);
    line(knob_room.left(), x, c.accent);
    let radius = if response.hovered() || response.dragged() {
        8.0
    } else {
        7.0
    };
    painter.circle(pos2(x, y), radius, c.text, Stroke::new(2.0, c.accent));
    response.on_hover_cursor(CursorIcon::PointingHand)
}
