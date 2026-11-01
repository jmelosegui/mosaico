//! The Play button under the curve.

use eframe::egui::{
    Color32, CursorIcon, Rect, Response, Role, Sense, Stroke, StrokeKind, Ui, WidgetInfo, vec2,
};

use crate::icons::{self, Icon};
use crate::theme::{self, Colors};

/// An outlined button with the lightning icon and "Play".
pub fn play_button(ui: &mut Ui, c: &Colors) -> Response {
    let font = theme::bold(12.0);
    let galley = ui.painter().layout_no_wrap("Play".to_owned(), font, c.text);
    let size = vec2(12.0 + 14.0 + 6.0 + galley.size().x + 12.0, 26.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Play"));
    let painter = ui.painter();
    let fill = if response.hovered() {
        c.hover
    } else {
        Color32::TRANSPARENT
    };
    painter.rect(
        rect,
        5.0,
        fill,
        Stroke::new(1.0, c.line),
        StrokeKind::Inside,
    );
    let icon = Rect::from_center_size(rect.left_center() + vec2(19.0, 0.0), vec2(14.0, 14.0));
    icons::paint(painter, icon, Icon::Animations, c.text, 1.5);
    let at = rect.left_center() + vec2(32.0, -galley.size().y / 2.0);
    painter.galley_with_override_text_color(at, galley, c.text);
    response.on_hover_cursor(CursorIcon::PointingHand)
}
