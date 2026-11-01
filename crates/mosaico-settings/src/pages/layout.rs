//! The Layout page: how windows are tiled and hidden, from `config.toml`.

use eframe::egui::{CursorIcon, Rect, Response, Role, Sense, Stroke, StrokeKind, Ui};
use eframe::egui::{WidgetInfo, pos2, vec2};
use mosaico_core::config::schema::{self, SettingKind, SettingValue};

use super::{Entry, PageContext, Row, Section};
use crate::model::Settings;
use crate::theme::{self, Colors};

pub const SECTIONS: &[Section] = &[
    ("Default layout", &[Entry::Bare(DEFAULT, cards)]),
    (
        "Spacing",
        &[Entry::Setting("layout.gap"), Entry::Setting("layout.ratio")],
    ),
    ("Workspace switching", &[Entry::Setting("layout.hiding")]),
];

const DEFAULT: &str = "layout.default";

/// Space between two cards.
const CARD_GAP: f32 = 12.0;

/// One card for each layout, picturing it. Clicking a card picks it.
fn cards(ui: &mut Ui, cx: &PageContext, _: &Row, edit: &mut Settings) {
    let setting = schema::find(DEFAULT).unwrap_or_else(|| panic!("no setting {DEFAULT}"));
    let (SettingKind::Choice(options), SettingValue::Text(chosen)) =
        (setting.kind, edit.get(DEFAULT))
    else {
        return;
    };
    let mut chosen = chosen.clone();
    let count = options.len() as f32;
    let width = (ui.available_width() - CARD_GAP * (count - 1.0)) / count;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = CARD_GAP;
        for (name, label) in options {
            if card(ui, cx.c, name, label, width, chosen == *name).clicked() {
                chosen = (*name).to_owned();
            }
        }
    });
    edit.set(DEFAULT, SettingValue::Text(chosen));
}

fn card(ui: &mut Ui, c: &Colors, name: &str, label: &str, width: f32, on: bool) -> Response {
    let picture = width - 20.0;
    let size = vec2(width, 10.0 + picture * 0.55 + 36.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    response.widget_info(|| WidgetInfo::selected(Role::RadioButton, true, on, label));
    if !ui.is_rect_visible(rect) {
        return response;
    }
    let painter = ui.painter();
    let stroke = if on {
        Stroke::new(2.0, c.accent)
    } else if response.hovered() {
        Stroke::new(1.0, c.faint_text)
    } else {
        Stroke::new(1.0, c.line)
    };
    painter.rect(rect, 8.0, c.card, stroke, StrokeKind::Inside);
    let frame = Rect::from_min_size(rect.min + vec2(10.0, 10.0), vec2(picture, picture * 0.55));
    painter.rect_filled(frame, 6.0, c.page);
    let area = frame.shrink(6.0);
    for (i, &(x0, y0, x1, y1)) in tiles(name).iter().enumerate() {
        let tile = Rect::from_min_max(
            pos2(
                area.left() + area.width() * x0,
                area.top() + area.height() * y0,
            ),
            pos2(
                area.left() + area.width() * x1,
                area.top() + area.height() * y1,
            ),
        )
        .shrink(3.0);
        let fill = if on && i == 0 { c.accent } else { c.raised };
        painter.rect_filled(tile, 3.0, fill);
    }
    let font = if on {
        theme::bold(13.0)
    } else {
        theme::regular(13.0)
    };
    let galley = painter.layout_no_wrap(label.to_owned(), font, c.text);
    let at = pos2(rect.left() + 14.0, rect.bottom() - 26.0);
    painter.galley_with_override_text_color(at, galley, c.text);
    response.on_hover_cursor(CursorIcon::PointingHand)
}

/// The windows of a layout as fractions of the screen, the main window
/// first.
fn tiles(name: &str) -> &'static [(f32, f32, f32, f32)] {
    const THIRD: f32 = 1.0 / 3.0;
    match name {
        "vertical-stack" => &[
            (0.0, 0.0, 0.5, 1.0),
            (0.5, 0.0, 1.0, THIRD),
            (0.5, THIRD, 1.0, 2.0 * THIRD),
            (0.5, 2.0 * THIRD, 1.0, 1.0),
        ],
        "three-column" => &[
            (0.25, 0.0, 0.75, 1.0),
            (0.0, 0.0, 0.25, 0.5),
            (0.0, 0.5, 0.25, 1.0),
            (0.75, 0.0, 1.0, 1.0),
        ],
        _ => &[
            (0.0, 0.0, 0.5, 1.0),
            (0.5, 0.0, 1.0, 0.5),
            (0.5, 0.5, 0.75, 1.0),
            (0.75, 0.5, 1.0, 1.0),
        ],
    }
}
