//! A row of choices, one of them selected.

use eframe::egui::{Align2, Frame, Margin, Response, Role, Sense, Stroke, Ui, WidgetInfo, vec2};

use crate::theme::{self, Colors};

/// Shows `options` side by side, with their labels, and selects the one
/// clicked. `changed` outlines the control in the accent color.
pub fn segmented<T: PartialEq + Copy>(
    ui: &mut Ui,
    c: &Colors,
    options: &[(T, &str)],
    selected: &mut T,
    changed: bool,
) -> Response {
    let stroke = if changed {
        Stroke::new(1.0, c.accent)
    } else {
        Stroke::NONE
    };
    let frame = Frame::new()
        .fill(c.control)
        .stroke(stroke)
        .corner_radius(6)
        .inner_margin(Margin::same(2));
    let inner = frame.show(ui, |ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        ui.horizontal(|ui| {
            let mut clicked = None;
            // A right-to-left parent would otherwise list the options backwards.
            let mut ordered = options.to_vec();
            if ui.layout().prefer_right_to_left() {
                ordered.reverse();
            }
            for (value, label) in ordered {
                if segment(ui, c, label, value == *selected).clicked() {
                    clicked = Some(value);
                }
            }
            clicked
        })
        .inner
    });
    let mut response = inner.response;
    if let Some(value) = inner.inner
        && value != *selected
    {
        *selected = value;
        response.mark_changed();
    }
    response
}

fn segment(ui: &mut Ui, c: &Colors, label: &str, on: bool) -> Response {
    let font = if on {
        theme::bold(12.0)
    } else {
        theme::regular(12.0)
    };
    let galley = ui.painter().layout_no_wrap(label.into(), font, c.text);
    let size = galley.size() + vec2(20.0, 8.0);
    let (rect, response) = ui.allocate_exact_size(vec2(size.x, size.y.max(22.0)), Sense::click());
    response.widget_info(|| WidgetInfo::selected(Role::RadioButton, true, on, label));
    let fill = if on {
        c.raised
    } else if response.hovered() {
        c.hover.gamma_multiply(0.6)
    } else {
        eframe::egui::Color32::TRANSPARENT
    };
    let painter = ui.painter();
    painter.rect_filled(rect, 4.0, fill);
    let color = if on || response.hovered() {
        c.text
    } else {
        c.dim_text
    };
    let pos = Align2::CENTER_CENTER
        .anchor_size(rect.center(), galley.size())
        .min;
    painter.galley_with_override_text_color(pos, galley, color);
    response
}
