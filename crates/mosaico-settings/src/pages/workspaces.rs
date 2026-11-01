//! The Workspaces page: how workspaces switch and what each one tiles
//! with, from `config.toml`.

use eframe::egui::{Align, Align2, Layout, Rect, RichText, Sense, Ui, UiBuilder, pos2, vec2};
use mosaico_core::config::schema::{self, SettingKind, SettingValue};

use super::{Entry, PageContext, Row, Section};
use crate::model::Settings;
use crate::theme::{self, Colors};
use crate::widgets;

pub const SECTIONS: &[Section] = &[
    (
        "Switching",
        &[
            Entry::Setting("workspaces.mode"),
            Entry::Setting("workspaces.wrap"),
        ],
    ),
    (
        "Layout per workspace",
        &[Entry::Bare("workspaces.layouts.1", grid)],
    ),
];

const HELP: &str =
    "Workspaces without a layout of their own use the default layout from the Layout page.";

/// Workspaces down each column of the grid.
const PER_COLUMN: usize = 4;

const CELL_HEIGHT: f32 = 44.0;

/// The layout of each workspace, two columns of cells with the number of
/// the workspace, its name and a drop-down to pick its layout.
fn grid(ui: &mut Ui, cx: &PageContext, _: &Row, edit: &mut Settings) {
    let c = cx.c;
    ui.label(
        RichText::new(HELP)
            .font(theme::regular(11.0))
            .color(c.dim_text),
    );
    ui.add_space(6.0);
    let default = default_layout(edit);
    widgets::card(ui, c, |ui| {
        let width = ui.available_width() / 2.0;
        for row in 0..PER_COLUMN {
            if row > 0 {
                widgets::divider(ui, c);
            }
            let (area, _) = ui.allocate_exact_size(vec2(2.0 * width, CELL_HEIGHT), Sense::hover());
            let top = area.min;
            for column in 0..2 {
                let n = column * PER_COLUMN + row + 1;
                let at = pos2(top.x + column as f32 * width, top.y);
                let rect = Rect::from_min_size(at, vec2(width, CELL_HEIGHT));
                cell(ui, cx, rect, n, &default, edit);
            }
            let line = pos2(top.x + width, top.y);
            ui.painter()
                .line_segment([line, line + vec2(0.0, CELL_HEIGHT)], (1.0, c.line));
        }
    });
}

/// The name of the layout workspaces without their own use.
fn default_layout(edit: &Settings) -> String {
    let key = "layout.default";
    let setting = schema::find(key).unwrap_or_else(|| panic!("no setting {key}"));
    let (SettingKind::Choice(options), SettingValue::Text(chosen)) = (setting.kind, edit.get(key))
    else {
        return String::new();
    };
    options
        .iter()
        .find(|(name, _)| name == chosen)
        .map_or_else(|| chosen.clone(), |(_, label)| (*label).to_owned())
}

/// The cell of workspace `n` in `rect`.
fn cell(ui: &mut Ui, cx: &PageContext, rect: Rect, n: usize, default: &str, edit: &mut Settings) {
    let key = format!("workspaces.layouts.{n}");
    let setting = schema::find(&key).unwrap_or_else(|| panic!("no setting {key}"));
    let (SettingKind::Choice(options), SettingValue::Text(chosen)) =
        (setting.kind, edit.get(&key).clone())
    else {
        return;
    };
    let default_label = format!("Default ({default})");
    let options: Vec<_> = options
        .iter()
        .map(|&(name, label)| {
            (
                name,
                if name.is_empty() {
                    &*default_label
                } else {
                    label
                },
            )
        })
        .collect();
    let mut picked = options
        .iter()
        .find(|(name, _)| *name == chosen)
        .map_or("", |(name, _)| *name);
    let own = !chosen.is_empty();
    let unsaved = edit.get(&key) != cx.saved.get(&key);
    let builder = UiBuilder::new()
        .max_rect(rect.shrink2(vec2(16.0, 9.0)))
        .layout(Layout::left_to_right(Align::Center));
    let mut ui = ui.new_child(builder);
    let ui = &mut ui;
    ui.spacing_mut().item_spacing.x = 12.0;
    number(ui, cx.c, n, own);
    let label = RichText::new(setting.label).font(theme::regular(13.0));
    ui.label(label);
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        let state = (unsaved, !own);
        let width = 168.0;
        widgets::dimmed_dropdown(ui, cx.c, setting.label, width, &options, &mut picked, state);
    });
    if picked != chosen {
        edit.set(&key, SettingValue::Text(picked.to_owned()));
    }
}

/// A square holding the number of a workspace, lit when it has a layout of
/// its own.
fn number(ui: &mut Ui, c: &Colors, n: usize, own: bool) {
    let (rect, _) = ui.allocate_exact_size(vec2(24.0, 24.0), Sense::hover());
    let (fill, text) = if own {
        (c.selected, c.accent)
    } else {
        (c.control, c.dim_text)
    };
    let painter = ui.painter();
    painter.rect_filled(rect, 5.0, fill);
    let galley = painter.layout_no_wrap(n.to_string(), theme::bold(12.0), text);
    let at = Align2::CENTER_CENTER
        .anchor_size(rect.center(), galley.size())
        .min;
    painter.galley_with_override_text_color(at, galley, text);
}
