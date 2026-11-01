//! The tools over the bindings: a filter and the button that restores
//! the default shortcuts.

use eframe::egui::{
    Align, Align2, CursorIcon, Frame, Layout, Margin, Rect, Response, Role, Sense, Stroke,
    StrokeKind, TextEdit, Ui, WidgetInfo, vec2,
};

use crate::icons::{self, Icon};
use crate::theme::{self, Colors};

const HEIGHT: f32 = 26.0;

/// Draws the filter, editing `filter`, and the Restore defaults button.
/// Returns whether the button was clicked.
pub fn draw(ui: &mut Ui, c: &Colors, filter: &mut String) -> bool {
    let mut restore = false;
    ui.add_space(4.0);
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), HEIGHT),
        Layout::right_to_left(Align::Center),
        |ui| {
            restore = restore_button(ui, c).clicked();
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                filter_field(ui, c, filter);
            });
        },
    );
    ui.add_space(16.0);
    restore
}

fn filter_field(ui: &mut Ui, c: &Colors, filter: &mut String) {
    let response = Frame::new()
        .fill(c.control)
        .corner_radius(5)
        .inner_margin(Margin::symmetric(8, 0))
        .show(ui, |ui| {
            ui.set_height(HEIGHT);
            ui.horizontal_centered(|ui| {
                let (rect, _) = ui.allocate_exact_size(vec2(14.0, 14.0), Sense::hover());
                icons::paint(ui.painter(), rect, Icon::Search, c.faint_text, 1.5);
                let field = TextEdit::singleline(filter)
                    .frame(Frame::NONE)
                    .align(Align2::LEFT_CENTER)
                    .hint_text("Filter by action or key")
                    .font(theme::regular(12.0))
                    .desired_width(230.0);
                ui.add(field)
            })
            .inner
        })
        .inner;
    response.widget_info(|| WidgetInfo::labeled(Role::TextInput, true, "Filter keybindings"));
}

fn restore_button(ui: &mut Ui, c: &Colors) -> Response {
    let text = "Restore defaults";
    let font = theme::bold(12.0);
    let galley = ui.painter().layout_no_wrap(text.to_owned(), font, c.text);
    let size = vec2(12.0 + 14.0 + 6.0 + galley.size().x + 12.0, HEIGHT);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, text));
    if ui.is_rect_visible(rect) {
        let fill = if response.hovered() { c.hover } else { c.card };
        let stroke = Stroke::new(1.0, c.line);
        ui.painter()
            .rect(rect, 5.0, fill, stroke, StrokeKind::Inside);
        let icon =
            Rect::from_center_size(rect.left_center() + vec2(12.0 + 7.0, 0.0), vec2(14.0, 14.0));
        icons::paint(ui.painter(), icon, Icon::Reset, c.text, 1.5);
        let at = rect.left_center() + vec2(12.0 + 14.0 + 6.0, -galley.size().y / 2.0);
        ui.painter().galley(at, galley, c.text);
    }
    response.on_hover_cursor(CursorIcon::PointingHand)
}
