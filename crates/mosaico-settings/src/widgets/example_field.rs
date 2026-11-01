//! A text field that shows, at its far end, what its value makes.

use eframe::egui::{
    Align, Align2, Frame, Layout, Margin, Response, RichText, Role, TextEdit, Ui, WidgetInfo,
};

use crate::theme::{self, Colors};

/// A line of text `width` points wide with `example` after it, such as a
/// time format and the time it writes. `label` names it for assistive
/// technology.
pub fn example_field(
    ui: &mut Ui,
    c: &Colors,
    label: &str,
    width: f32,
    value: &mut String,
    example: &str,
) -> Response {
    let example = RichText::new(example)
        .font(theme::regular(11.0))
        .color(c.faint_text);
    let response = Frame::new()
        .fill(c.control)
        .corner_radius(5)
        .inner_margin(Margin::symmetric(6, 0))
        .show(ui, |ui| {
            ui.set_height(26.0);
            ui.set_width(width);
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                let shown = ui.painter().layout_no_wrap(
                    example.text().to_owned(),
                    theme::regular(11.0),
                    c.faint_text,
                );
                let gap = ui.spacing().item_spacing.x;
                let field = TextEdit::singleline(value)
                    .frame(Frame::NONE)
                    .align(Align2::LEFT_CENTER)
                    .font(theme::regular(12.0))
                    .desired_width((width - shown.size().x - gap).max(24.0));
                let response = ui.add(field);
                ui.label(example);
                response
            })
            .inner
        })
        .inner;
    response.widget_info(|| WidgetInfo::labeled(Role::TextInput, true, label));
    if response.changed() {
        // The example follows the value from the next frame on.
        ui.ctx().request_repaint();
    }
    response
}
