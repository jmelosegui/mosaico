//! The button of a color picker: the color's swatch and name.

use eframe::egui::{Rect, Stroke, StrokeKind, Ui, pos2, vec2};
use mosaico_core::config::Color;

use super::color::paint;
use crate::icons::{self, Icon};
use crate::theme::{self, Colors};

pub(super) struct Button<'a> {
    pub rect: Rect,
    pub text: &'a str,
    pub dim: bool,
    pub swatch: Option<Color>,
    pub outlined: bool,
    pub open: bool,
}

pub(super) fn paint_button(ui: &Ui, c: &Colors, button: &Button) {
    let rect = button.rect;
    let painter = ui.painter();
    let stroke = if button.outlined {
        Stroke::new(1.0, c.accent)
    } else {
        Stroke::NONE
    };
    painter.rect(rect, 5.0, c.control, stroke, StrokeKind::Inside);
    let chip = Rect::from_center_size(rect.left_center() + vec2(17.0, 0.0), vec2(14.0, 14.0));
    paint_chip(ui, c, chip, button.swatch);
    let color = if button.dim { c.faint_text } else { c.text };
    let galley = painter.layout_no_wrap(button.text.to_owned(), theme::regular(12.0), color);
    let at = pos2(rect.left() + 32.0, rect.center().y - galley.size().y / 2.0);
    painter.galley_with_override_text_color(at, galley, color);
    let arrow = Rect::from_center_size(rect.right_center() - vec2(15.0, 0.0), vec2(14.0, 14.0));
    let icon = if button.open { Icon::Up } else { Icon::Chevron };
    icons::paint(painter, arrow, icon, c.dim_text, 1.5);
}

fn paint_chip(ui: &Ui, c: &Colors, rect: Rect, color: Option<Color>) {
    let painter = ui.painter();
    match color {
        Some(color) => {
            painter.rect(
                rect,
                3,
                paint(color),
                Stroke::new(1.0, c.line),
                StrokeKind::Inside,
            );
        }
        None => {
            painter.rect_stroke(rect, 3, Stroke::new(1.0, c.faint_text), StrokeKind::Inside);
            painter.line_segment(
                [rect.left_bottom(), rect.right_top()],
                Stroke::new(1.0, c.faint_text),
            );
        }
    }
}
