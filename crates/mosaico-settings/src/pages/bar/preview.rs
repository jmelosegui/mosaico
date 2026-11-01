//! The bar as it is drawn, at half size, from the settings being edited.

use eframe::egui::{Frame, Margin, ScrollArea, Sense, Stroke, Ui, vec2};
use mosaico_core::bar::sample_text;
use mosaico_core::config::{BarWidget, BarZone, Theme, WidgetKind};

use super::columns::glyphs::drawing;
use super::look::{Look, widget_color};
use super::pills::{Pill, draw, width};
use super::state::BarState;
use crate::icons::Icon;
use crate::theme::Colors;

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

/// How wide the bar is drawn, given how wide its `left` and `right`
/// widgets are and the `room` the page has for it: the whole room, or
/// more when the widgets need it, so they never overlap.
fn bar_width(left: f32, right: f32, look: &Look, room: f32) -> f32 {
    room.max(left + right + look.gap + look.edge * 2.0)
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
            let side = |zone| -> Vec<Vec<Pill>> {
                let widgets = edit.widgets.zone(zone).iter().filter(|w| w.enabled);
                widgets.map(|widget| sample(widget, &look)).collect()
            };
            let (left, right) = (side(BarZone::Left), side(BarZone::Right));
            let span = |groups: &[Vec<Pill>]| {
                let total: f32 = groups.iter().map(|g| width(ui.painter(), &look, g)).sum();
                total + look.gap * groups.len().saturating_sub(1) as f32
            };
            let room = ui.available_width();
            let bar = bar_width(span(&left), span(&right), &look, room);
            ScrollArea::horizontal()
                .id_salt("bar preview")
                .show(ui, |ui| {
                    let (rect, _) = ui.allocate_exact_size(vec2(bar, look.height), Sense::hover());
                    let painter = ui.painter();
                    painter.rect_filled(rect, 0.0, look.background);
                    let mut x = rect.left() + look.edge;
                    for pills in &left {
                        x += draw(painter, rect, &look, pills, x) + look.gap;
                    }
                    let mut x = rect.right() - look.edge;
                    for pills in &right {
                        x -= width(painter, &look, pills);
                        draw(painter, rect, &look, pills, x);
                        x -= look.gap;
                    }
                });
        })
        .response
        .widget_info(|| {
            eframe::egui::WidgetInfo::labeled(eframe::egui::Role::Image, true, "Bar preview")
        });
    ui.add_space(16.0);
}

#[cfg(test)]
#[path = "preview_tests.rs"]
mod tests;
