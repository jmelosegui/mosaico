//! The bar as it is drawn, at half size, from the settings being edited.

use eframe::egui::{
    Color32, Frame, Margin, Painter, Rect, Sense, Stroke, StrokeKind, Ui, pos2, vec2,
};
use mosaico_core::config::schema::SettingValue;
use mosaico_core::config::{BarWidget, BarZone, Theme, WidgetKind};

use super::state::BarState;
use crate::icons::{self, Icon};
use crate::theme::{self, Colors};

/// One pill of the preview.
struct Pill {
    icon: Option<Icon>,
    text: String,
    fill: Color32,
    color: Color32,
}

/// What the preview draws for the widget, or nothing for the widgets
/// that only appear on some condition.
fn sample(widget: &BarWidget, look: &Look) -> Vec<Pill> {
    let pill = |icon: Icon, text: &str| Pill {
        icon: Some(icon),
        text: text.to_owned(),
        fill: look.pill,
        color: widget_color(widget, look),
    };
    match widget.kind {
        WidgetKind::Workspaces => (1..=4)
            .map(|n| {
                let on = n == 1;
                Pill {
                    icon: None,
                    text: n.to_string(),
                    fill: if on { look.active } else { look.pill },
                    color: if on {
                        look.active_text
                    } else {
                        look.inactive_text
                    },
                }
            })
            .collect(),
        WidgetKind::Layout => vec![pill(Icon::Grid, "BSP")],
        WidgetKind::ActiveWindow => vec![pill(Icon::Window, "mosaico settings")],
        WidgetKind::Clock => vec![pill(Icon::Clock, "12:57:03")],
        WidgetKind::Date => vec![pill(Icon::Calendar, "Saturday 31 October 2026")],
        WidgetKind::Ram => vec![pill(Icon::Memory, "48%")],
        WidgetKind::Cpu => vec![pill(Icon::Processor, "12%")],
        WidgetKind::Update | WidgetKind::Media | WidgetKind::Paused => Vec::new(),
    }
}

/// The colors and measures the bar is drawn with.
struct Look {
    background: Color32,
    text: Color32,
    pill: Color32,
    border: Color32,
    active: Color32,
    active_text: Color32,
    inactive_text: Color32,
    theme: Theme,
    height: f32,
    font: f32,
    edge: f32,
    pad: f32,
    radius: f32,
    border_width: f32,
    gap: f32,
    workspace_gap: f32,
}

impl Look {
    fn of(edit: &BarState, theme: Theme) -> Self {
        let color = |key: &str| {
            let Some(SettingValue::Text(value)) = edit.get(key).cloned() else {
                return Color32::TRANSPARENT;
            };
            let fallback = theme.default_color(key).unwrap_or_default();
            theme
                .swatch(&value, &fallback)
                .map_or(Color32::TRANSPARENT, |c| Color32::from_rgb(c.r, c.g, c.b))
        };
        let number = |key: &str, scale: f32| match edit.get(key) {
            Some(SettingValue::Int(n)) => *n as f32 * scale,
            _ => 0.0,
        };
        let opacity = number("background_opacity", 0.01).clamp(0.0, 1.0);
        Self {
            background: color("colors.background").gamma_multiply(opacity),
            text: color("colors.foreground"),
            pill: color("colors.widget_background"),
            border: color("colors.pill_border"),
            active: color("colors.active_workspace"),
            active_text: color("colors.active_workspace_text"),
            inactive_text: color("colors.inactive_workspace"),
            theme,
            height: number("height", 0.5).clamp(16.0, 48.0),
            font: number("font_size", 0.5).clamp(8.0, 14.0),
            edge: number("padding", 0.5),
            pad: number("pill_padding", 0.5),
            radius: number("pill_radius", 0.5),
            border_width: number("pill_border_width", 0.5),
            gap: number("item_gap", 0.5),
            workspace_gap: number("workspace_gap", 0.5),
        }
    }
}

fn widget_color(widget: &BarWidget, look: &Look) -> Color32 {
    look.theme
        .swatch(&widget.color, "")
        .map_or(look.text, |c| Color32::from_rgb(c.r, c.g, c.b))
}

/// Draws the bar with the widgets of `edit` that are on.
pub fn show(ui: &mut Ui, c: &Colors, edit: &BarState, theme: Theme) {
    let look = Look::of(edit, theme);
    Frame::new()
        .fill(c.sidebar)
        .stroke(Stroke::new(1.0, c.line))
        .corner_radius(8)
        .inner_margin(Margin::same(10))
        .show(ui, |ui| {
            let (rect, _) =
                ui.allocate_exact_size(vec2(ui.available_width(), look.height), Sense::hover());
            let painter = ui.painter();
            painter.rect_filled(rect, 0.0, look.background);
            let mut left = rect.left() + look.edge;
            for widget in edit
                .widgets
                .zone(BarZone::Left)
                .iter()
                .filter(|w| w.enabled)
            {
                left += draw(painter, rect, &look, &sample(widget, &look), left) + look.gap;
            }
            let mut right = rect.right() - look.edge;
            for widget in edit
                .widgets
                .zone(BarZone::Right)
                .iter()
                .filter(|w| w.enabled)
            {
                let pills = sample(widget, &look);
                right -= width(painter, &look, &pills);
                draw(painter, rect, &look, &pills, right);
                right -= look.gap;
            }
        })
        .response
        .widget_info(|| {
            eframe::egui::WidgetInfo::labeled(eframe::egui::Role::Image, true, "Bar preview")
        });
    ui.add_space(16.0);
}

fn pill_width(painter: &Painter, look: &Look, pill: &Pill) -> f32 {
    let galley = painter.layout_no_wrap(pill.text.clone(), theme::bold(look.font), pill.color);
    galley.size().x + look.pad * 2.0 + icon_room(look, pill)
}

/// The width of a pill's icon and the gap after it, as the bar leaves.
fn icon_room(look: &Look, pill: &Pill) -> f32 {
    if pill.icon.is_some() {
        look.font + ICON_GAP
    } else {
        0.0
    }
}

const ICON_GAP: f32 = 3.0;

fn width(painter: &Painter, look: &Look, pills: &[Pill]) -> f32 {
    let total: f32 = pills.iter().map(|p| pill_width(painter, look, p)).sum();
    total + look.workspace_gap * pills.len().saturating_sub(1) as f32
}

/// Draws `pills` from `x`, returning how wide they are.
fn draw(painter: &Painter, bar: Rect, look: &Look, pills: &[Pill], x: f32) -> f32 {
    let mut x = x;
    let start = x;
    let height = (look.font + 8.0).min(bar.height());
    for pill in pills {
        let w = pill_width(painter, look, pill);
        let rect = Rect::from_min_size(pos2(x, bar.center().y - height / 2.0), vec2(w, height));
        let stroke = Stroke::new(look.border_width.min(2.0), look.border);
        painter.rect(rect, look.radius, pill.fill, stroke, StrokeKind::Inside);
        let galley = painter.layout_no_wrap(pill.text.clone(), theme::bold(look.font), pill.color);
        let room = icon_room(look, pill);
        let at = rect.center() - galley.size() / 2.0 + vec2(room / 2.0, 0.0);
        if let Some(icon) = pill.icon {
            let glyph = Rect::from_min_size(
                pos2(at.x - room, rect.center().y - look.font / 2.0),
                vec2(look.font, look.font),
            );
            icons::paint(painter, glyph, icon, pill.color, 1.3);
        }
        painter.galley_with_override_text_color(at, galley, pill.color);
        x += w + look.workspace_gap;
    }
    (x - look.workspace_gap - start).max(0.0)
}

#[cfg(test)]
#[path = "preview_tests.rs"]
mod tests;
