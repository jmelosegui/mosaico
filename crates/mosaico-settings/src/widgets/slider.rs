//! A number on a track, with the number field beside it.

use eframe::egui::{
    Align, CursorIcon, Layout, Rect, Response, Sense, Stroke, Ui, UiBuilder, WidgetInfo,
    emath::Numeric, pos2, vec2,
};

use super::fields::{NUMBER_WIDTH, number};
use crate::theme::Colors;

/// A track for a number in `range`, then the number and its `unit`,
/// which can also be typed. `label` names both for assistive
/// technology.
#[allow(clippy::too_many_arguments)]
pub fn slider<N: Numeric>(
    ui: &mut Ui,
    c: &Colors,
    label: &str,
    value: &mut N,
    range: std::ops::RangeInclusive<N>,
    speed: f64,
    unit: &str,
    changed: bool,
) -> Response {
    let size = vec2(SLIDER_WIDTH + 10.0 + NUMBER_WIDTH, 26.0);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let builder = UiBuilder::new()
        .max_rect(rect)
        .layout(Layout::left_to_right(Align::Center));
    ui.scope_builder(builder, |ui| {
        ui.spacing_mut().item_spacing.x = 10.0;
        let track = track(ui, c, label, value, &range);
        let field = number(ui, c, label, value, range, speed, unit, changed);
        track | field
    })
    .inner
}

/// How wide the track of a [`slider`] is.
const SLIDER_WIDTH: f32 = 150.0;

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
    let ring = if response.hovered() || response.dragged() {
        c.text
    } else {
        c.accent.gamma_multiply(0.6)
    };
    painter.circle(pos2(x, y), 7.0, c.accent, Stroke::new(2.0, ring));
    response.on_hover_cursor(CursorIcon::PointingHand)
}
