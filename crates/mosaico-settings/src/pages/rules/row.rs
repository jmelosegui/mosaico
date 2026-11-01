//! One rule as a compact row: its chips, what it does to a window, and
//! the buttons of a rule of the user's.

use eframe::egui::{
    Align, Color32, CursorIcon, Frame, Layout, Margin, Rect, Response, Role, Sense, Stroke,
    StrokeKind, Ui, WidgetInfo, vec2,
};
use mosaico_core::config::{RuleAction, WindowRule};

use crate::icons::{self, Icon};
use crate::theme::{self, Colors};
use crate::widgets;

/// What the user asked of a row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ask {
    /// Open or close the editor of the rule.
    Edit,
    Delete,
    /// Copy the community rule to the user's.
    Override,
}

/// What a row reports back.
pub struct Shown {
    pub ask: Option<Ask>,
    /// The handle to drag the row by, on a rule of the user's.
    pub grip: Option<Response>,
    pub rect: Rect,
}

/// A rule of the user's: a grip, chips, a badge and the edit and delete
/// buttons. It is filled while `editing`.
pub fn user(ui: &mut Ui, c: &Colors, n: usize, rule: &WindowRule, editing: bool) -> Shown {
    let fill = if editing {
        c.selected
    } else {
        Color32::TRANSPARENT
    };
    let mut ask = None;
    let mut grip = None;
    let shown = Frame::new()
        .fill(fill)
        .inner_margin(Margin {
            left: 12,
            right: 12,
            top: 9,
            bottom: 9,
        })
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.x = 10.0;
            ui.horizontal(|ui| {
                grip = Some(handle(ui, c, n));
                chips(ui, c, rule);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let trash = widgets::icon_button(
                        ui,
                        c,
                        Icon::Trash,
                        &format!("Delete rule {n}"),
                        c.faint_text,
                    );
                    let color = if editing { c.accent } else { c.faint_text };
                    let label = format!("Edit rule {n}");
                    let edit = widgets::icon_button(ui, c, Icon::Edit, &label, color);
                    decision(ui, c, rule.action);
                    if trash.clicked() {
                        ask = Some(Ask::Delete);
                    } else if edit.clicked() {
                        ask = Some(Ask::Edit);
                    }
                });
            });
        });
    Shown {
        ask,
        grip,
        rect: shown.response.rect,
    }
}

/// A community rule, which cannot be edited but can be overridden.
pub fn community(ui: &mut Ui, c: &Colors, n: usize, rule: &WindowRule) -> Option<Ask> {
    let mut ask = None;
    Frame::new()
        .inner_margin(Margin {
            left: 16,
            right: 12,
            top: 9,
            bottom: 9,
        })
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.x = 10.0;
            // As tall as the Override button, with no icon button here.
            ui.spacing_mut().interact_size.y = 21.0;
            ui.horizontal(|ui| {
                chips(ui, c, rule);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let label = format!("Override community rule {n}");
                    if override_button(ui, c, &label).clicked() {
                        ask = Some(Ask::Override);
                    }
                    decision(ui, c, rule.action);
                });
            });
        });
    ask
}

/// What a rule matches on, one chip each.
pub fn chips(ui: &mut Ui, c: &Colors, rule: &WindowRule) {
    ui.spacing_mut().item_spacing.x = 6.0;
    let matches = [
        ("class is", &rule.match_class),
        ("title contains", &rule.match_title),
        ("executable", &rule.match_path),
    ];
    for (key, value) in matches {
        let Some(value) = value else { continue };
        if key == "title contains" && value.is_empty() {
            widgets::chip(ui, c, Some("title is"), "empty");
        } else {
            widgets::chip(ui, c, Some(key), value);
        }
    }
}

fn decision(ui: &mut Ui, c: &Colors, action: RuleAction) {
    let color = match action {
        RuleAction::Tile => c.good,
        RuleAction::Float => c.accent,
        RuleAction::Ignore => c.dim_text,
    };
    widgets::badge(ui, super::editor::action_label(action), color);
}

fn handle(ui: &mut Ui, c: &Colors, n: usize) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(14.0, 20.0), Sense::drag());
    let label = format!("Move rule {n}");
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, &label));
    // Drawn larger than the box, so the dots stand apart as in the sketch.
    let glyph = Rect::from_center_size(rect.center(), vec2(19.0, 19.0));
    icons::paint(ui.painter(), glyph, Icon::Grip, c.faint_text, 1.4);
    response.on_hover_cursor(CursorIcon::Grab)
}

/// The primary button of the page: a plus, then `label`.
pub fn add_button(ui: &mut Ui, c: &Colors, label: &str) -> Response {
    let look = Look {
        fill: c.accent,
        hover: c.accent.gamma_multiply(0.9),
        line: None,
        text: c.on_accent,
    };
    small_button(ui, &look, Some(Icon::Plus), label, label)
}

/// A small button outlined in the line color, as Done.
pub fn outlined_button(ui: &mut Ui, c: &Colors, label: &str) -> Response {
    let look = Look {
        fill: c.control,
        hover: c.hover,
        line: Some(c.line),
        text: c.text,
    };
    small_button(ui, &look, None, label, label)
}

/// "Override" in the accent color, read out as `label`.
fn override_button(ui: &mut Ui, c: &Colors, label: &str) -> Response {
    let look = Look {
        fill: Color32::TRANSPARENT,
        hover: c.hover,
        line: None,
        text: c.accent,
    };
    small_button(ui, &look, None, "Override", label)
}

/// How a small button is painted.
struct Look {
    fill: Color32,
    hover: Color32,
    line: Option<Color32>,
    text: Color32,
}

/// A button of 3 points by 10 of margin around an optional icon and
/// `text`, in bold 12, read out as `label`.
fn small_button(ui: &mut Ui, look: &Look, icon: Option<Icon>, text: &str, label: &str) -> Response {
    let font = theme::bold(12.0);
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_owned(), font, look.text);
    let glyph = if icon.is_some() { 14.0 + 6.0 } else { 0.0 };
    let size = vec2(10.0 + glyph + galley.size().x + 10.0, galley.size().y + 6.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, label));
    let fill = if response.hovered() {
        look.hover
    } else {
        look.fill
    };
    let painter = ui.painter();
    painter.rect_filled(rect, 5.0, fill);
    if let Some(line) = look.line {
        let stroke = Stroke::new(1.0, line);
        painter.rect_stroke(rect, 5.0, stroke, StrokeKind::Inside);
    }
    if let Some(icon) = icon {
        let at = Rect::from_min_size(rect.left_center() + vec2(10.0, -7.0), vec2(14.0, 14.0));
        icons::paint(painter, at, icon, look.text, 2.0);
    }
    let at = rect.left_center() + vec2(10.0 + glyph, -galley.size().y / 2.0);
    painter.galley_with_override_text_color(at, galley, look.text);
    response.on_hover_cursor(CursorIcon::PointingHand)
}
