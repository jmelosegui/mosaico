//! The pills of the bar preview: how wide each is and how it is drawn.

use eframe::egui::{Color32, Painter, Rect, Stroke, StrokeKind, pos2, vec2};

use super::look::Look;
use crate::icons::{self, Icon};
use crate::theme;

/// One pill of the preview.
pub(super) struct Pill {
    pub(super) icon: Option<Icon>,
    pub(super) text: String,
    pub(super) fill: Color32,
    pub(super) color: Color32,
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

pub(super) fn width(painter: &Painter, look: &Look, pills: &[Pill]) -> f32 {
    let total: f32 = pills.iter().map(|p| pill_width(painter, look, p)).sum();
    total + look.workspace_gap * pills.len().saturating_sub(1) as f32
}

/// Draws `pills` from `x`, returning how wide they are.
pub(super) fn draw(painter: &Painter, bar: Rect, look: &Look, pills: &[Pill], x: f32) -> f32 {
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
