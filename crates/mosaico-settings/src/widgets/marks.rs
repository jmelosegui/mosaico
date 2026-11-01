//! Small marks inside rows: chips, badges, keycaps and icon buttons.

use eframe::egui::{
    Color32, CursorIcon, FontId, Frame, Margin, Rect, Response, RichText, Role, Sense, Stroke, Ui,
    WidgetInfo, pos2, vec2,
};

use crate::icons::{self, Icon};
use crate::theme::{self, Colors};

/// A rounded chip reading `label`, after a dim `key` when there is one,
/// as in "class is Notepad".
pub fn chip(ui: &mut Ui, c: &Colors, key: Option<&str>, label: &str) -> Response {
    let font = theme::regular(11.0);
    let mut parts = Vec::with_capacity(2);
    if let Some(key) = key {
        parts.push((key, c.faint_text));
    }
    parts.push((label, c.text));
    pill(ui, c.control, &font, &parts)
}

/// A chip in `color`, for a state such as "Tile" or "Don't tile".
pub fn badge(ui: &mut Ui, label: &str, color: Color32) -> Response {
    let fill = color.gamma_multiply(0.18);
    pill(ui, fill, &theme::bold(11.0), &[(label, color)])
}

/// Texts 5 points apart in a pill of 3 points by 9 of margin. It is
/// painted at its own size, so a row centers it instead of stretching it.
fn pill(ui: &mut Ui, fill: Color32, font: &FontId, parts: &[(&str, Color32)]) -> Response {
    let galleys: Vec<_> = parts
        .iter()
        .map(|(text, color)| {
            let text = (*text).to_owned();
            ui.painter().layout_no_wrap(text, font.clone(), *color)
        })
        .collect();
    let gaps = 5.0 * galleys.len().saturating_sub(1) as f32;
    let width = galleys.iter().map(|g| g.size().x).sum::<f32>() + gaps;
    let height = galleys.iter().map(|g| g.size().y).fold(0.0, f32::max);
    let (rect, response) = ui.allocate_exact_size(vec2(width + 18.0, height + 6.0), Sense::hover());
    ui.painter().rect_filled(rect, 10.0, fill);
    let mut x = rect.left() + 9.0;
    for (i, (galley, (text, _))) in galleys.into_iter().zip(parts).enumerate() {
        let at = pos2(x, rect.center().y - galley.size().y / 2.0);
        x += galley.size().x + 5.0;
        // Each text is read out on its own, as a label would be.
        let part = Rect::from_min_size(at, galley.size());
        let read = ui.interact(part, response.id.with(i), Sense::hover());
        read.widget_info(|| WidgetInfo::labeled(Role::Label, true, *text));
        ui.painter().galley(at, galley, Color32::PLACEHOLDER);
    }
    response
}

/// The keys of a shortcut, one cap each, outlined in the accent when
/// they are `changed`.
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
