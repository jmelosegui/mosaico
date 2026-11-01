//! Small marks inside rows: chips, badges, keycaps and icon buttons.

use eframe::egui::{
    Color32, CursorIcon, Frame, Margin, Rect, Response, RichText, Role, Sense, Stroke, Ui,
    WidgetInfo, vec2,
};

use crate::icons::{self, Icon};
use crate::theme::{self, Colors};

/// A rounded chip reading `label`, after a dim `key` when there is one,
/// as in "class is Notepad".
#[expect(dead_code, reason = "the page redesigns use these")]
pub fn chip(ui: &mut Ui, c: &Colors, key: Option<&str>, label: &str) -> Response {
    Frame::new()
        .fill(c.control)
        .corner_radius(10)
        .inner_margin(Margin::symmetric(9, 3))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.x = 5.0;
            if let Some(key) = key {
                let key = RichText::new(key).font(theme::regular(11.0));
                ui.label(key.color(c.faint_text));
            }
            ui.label(RichText::new(label).font(theme::regular(11.0)));
        })
        .response
}

/// A chip in `color`, for a state such as "Tile" or "Don't tile".
#[expect(dead_code, reason = "the page redesigns use these")]
pub fn badge(ui: &mut Ui, label: &str, color: Color32) -> Response {
    Frame::new()
        .fill(color.gamma_multiply(0.18))
        .corner_radius(10)
        .inner_margin(Margin::symmetric(9, 3))
        .show(ui, |ui| {
            let text = RichText::new(label).font(theme::bold(11.0)).color(color);
            ui.label(text);
        })
        .response
}

/// The keys of a shortcut, one cap each, outlined in the accent when
/// they are `changed`.
#[expect(dead_code, reason = "the page redesigns use these")]
pub fn keycaps(ui: &mut Ui, c: &Colors, keys: &[&str], changed: bool) -> Response {
    let stroke = Stroke::new(1.0, if changed { c.accent } else { c.line });
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        for key in keys {
            Frame::new()
                .fill(c.control)
                .stroke(stroke)
                .corner_radius(4)
                .inner_margin(Margin::symmetric(7, 2))
                .show(ui, |ui| {
                    ui.label(RichText::new(*key).font(theme::bold(11.0)));
                });
        }
    })
    .response
}

/// A square button showing `icon` in `color`, read out as `label`.
pub fn icon_button(ui: &mut Ui, c: &Colors, icon: Icon, label: &str, color: Color32) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(26.0, 26.0), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), label));
    if ui.is_rect_visible(rect) {
        if response.hovered() {
            ui.painter().rect_filled(rect, 5.0, c.hover);
        }
        let glyph = Rect::from_center_size(rect.center(), vec2(14.0, 14.0));
        icons::paint(ui.painter(), glyph, icon, color, 1.5);
    }
    response
        .on_hover_text(label)
        .on_hover_cursor(CursorIcon::PointingHand)
}

/// A box ticked when `on`, then `label`. Clicking either flips `on`.
pub fn checkbox(ui: &mut Ui, c: &Colors, label: &str, on: &mut bool) -> Response {
    let font = theme::regular(13.0);
    let galley = ui.painter().layout_no_wrap(label.to_owned(), font, c.text);
    let size = vec2(16.0 + 8.0 + galley.size().x, 20.0);
    let (rect, mut response) = ui.allocate_exact_size(size, Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    response.widget_info(|| WidgetInfo::selected(Role::CheckBox, true, *on, label));
    if !ui.is_rect_visible(rect) {
        return response;
    }
    let mark = Rect::from_min_size(rect.left_center() - vec2(0.0, 8.0), vec2(16.0, 16.0));
    let painter = ui.painter();
    if *on {
        painter.rect_filled(mark, 4.0, c.accent);
        let tick = mark.shrink(2.0);
        icons::paint(painter, tick, Icon::Check, c.on_accent, 2.0);
    } else {
        let line = if response.hovered() {
            c.dim_text
        } else {
            c.faint_text
        };
        let stroke = Stroke::new(1.0, line);
        painter.rect(
            mark,
            4.0,
            c.control,
            stroke,
            eframe::egui::StrokeKind::Inside,
        );
    }
    let text = rect.left_center() + vec2(24.0, -galley.size().y / 2.0);
    painter.galley(text, galley, c.text);
    response.on_hover_cursor(CursorIcon::PointingHand)
}

/// A borderless button in the accent color: a plus and `label`.
pub fn ghost_button(ui: &mut Ui, c: &Colors, label: &str) -> Response {
    let font = theme::bold(12.0);
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font, c.accent);
    let size = vec2(14.0 + 6.0 + galley.size().x + 16.0, 28.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), label));
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        if response.hovered() {
            painter.rect_filled(rect, 5.0, c.hover);
        }
        let glyph = Rect::from_center_size(rect.left_center() + vec2(15.0, 0.0), vec2(12.0, 12.0));
        icons::paint(painter, glyph, Icon::Plus, c.accent, 2.0);
        let at = rect.left_center() + vec2(28.0, -galley.size().y / 2.0);
        painter.galley(at, galley, c.accent);
    }
    response.on_hover_cursor(CursorIcon::PointingHand)
}
