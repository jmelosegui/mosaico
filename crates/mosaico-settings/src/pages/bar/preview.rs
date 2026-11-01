//! The bar as it is drawn, at half size, from the settings being edited.

use eframe::egui::{
    Color32, Frame, Margin, Painter, Rect, Sense, Stroke, StrokeKind, Ui, pos2, vec2,
};
use mosaico_core::bar::sample_text;
use mosaico_core::config::{BarWidget, BarZone, Theme, WidgetKind};

use super::columns::glyphs::drawing;
use super::look::{Look, widget_color};
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

/// What the preview draws for the widget: the drawing of its icon and
/// what the bar writes for it on a sample bar where every widget shows.
fn sample(widget: &BarWidget, look: &Look) -> Vec<Pill> {
    let pill = |icon: Option<Icon>, text: String| Pill {
        icon,
        text,
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
        WidgetKind::ActiveWindow => {
            vec![pill(Some(Icon::Window), "mosaico settings".into())]
        }
        _ => vec![pill(drawing(&widget.icon), sample_text(widget))],
    }
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
