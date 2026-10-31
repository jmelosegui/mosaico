//! An on/off switch.

use eframe::egui::{Response, Role, Sense, Stroke, Ui, WidgetInfo, vec2};

use crate::theme::Colors;

/// A switch for `on`, read out as `label`. Clicking it flips `on`.
pub fn toggle(ui: &mut Ui, c: &Colors, label: &str, on: &mut bool) -> Response {
    let (rect, mut response) = ui.allocate_exact_size(vec2(34.0, 20.0), Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    response.widget_info(|| WidgetInfo::selected(Role::Switch, true, *on, label));
    if !ui.is_rect_visible(rect) {
        return response;
    }
    let t = ui.ctx().animate_bool_responsive(response.id, *on);
    let fill = c.control.lerp_to_gamma(c.accent, t);
    let painter = ui.painter();
    painter.rect_filled(rect, 10.0, fill);
    if t < 1.0 {
        let line = if response.hovered() {
            c.faint_text
        } else {
            c.line
        };
        let stroke = Stroke::new(1.0, line.gamma_multiply(1.0 - t));
        painter.rect_stroke(rect, 10.0, stroke, eframe::egui::StrokeKind::Inside);
    }
    let x = rect.left() + 10.0 + (rect.width() - 20.0) * t;
    let knob = c.faint_text.lerp_to_gamma(c.on_accent, t);
    painter.circle_filled(eframe::egui::pos2(x, rect.center().y), 7.0, knob);
    response
}
