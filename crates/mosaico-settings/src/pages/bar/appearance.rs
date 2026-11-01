//! The Appearance tab: where the bar shows, its font, and its pills.

use eframe::egui::{
    Align2, Color32, CursorIcon, Frame, Margin, Rect, Response, Role, Sense, Stroke, Ui,
    WidgetInfo, vec2,
};
use mosaico_core::config::schema::{self, SettingKind, SettingValue};

use super::BarPage;
use crate::theme::{self, Colors};
use crate::widgets::{self, RowState};

/// The numbers of the grid of pills and spacing, three to a line.
const GRID: [[&str; 3]; 2] = [
    ["padding", "pill_padding", "pill_radius"],
    ["pill_border_width", "item_gap", "workspace_gap"],
];

/// The keys of the colors, in the order the file has them.
pub const COLORS: &[&str] = &[
    "colors.background",
    "colors.foreground",
    "colors.active_workspace",
    "colors.active_workspace_text",
    "colors.inactive_workspace",
    "colors.separator",
    "colors.accent",
    "colors.widget_background",
    "colors.pill_border",
];

/// The font styles, as the keys that switch them and how they are shown.
const STYLES: [(&str, &str, &str); 3] = [
    ("font_bold", "Bold", "B"),
    ("font_italic", "Italic", "I"),
    ("font_underline", "Underline", "U"),
];

impl BarPage {
    pub(super) fn appearance(&mut self, ui: &mut Ui, c: &Colors) {
        widgets::section(ui, c, "Bar", |ui| {
            self.setting(ui, c, "enabled");
            widgets::divider(ui, c);
            self.monitors_row(ui, c);
            widgets::divider(ui, c);
            self.setting(ui, c, "height");
            widgets::divider(ui, c);
            self.setting(ui, c, "background_opacity");
        });
        widgets::section(ui, c, "Font", |ui| {
            self.family(ui, c);
            widgets::divider(ui, c);
            self.setting(ui, c, "font_size");
            widgets::divider(ui, c);
            self.style_row(ui, c);
        });
        widgets::section(ui, c, "Pills and spacing", |ui| self.grid(ui, c));
        self.sections(ui, c, "", &[("Separator", &["separator"])]);
        self.sections(ui, c, "", &[("Colors", COLORS)]);
    }

    /// One checkbox per monitor. With none ticked the bar shows on all.
    fn monitors_row(&mut self, ui: &mut Ui, c: &Colors) {
        let count = self
            .edit
            .monitors
            .iter()
            .map(|m| m + 1)
            .max()
            .unwrap_or(0)
            .max(self.monitors);
        let state = RowState {
            unsaved: self.edit.monitors != self.saved.monitors,
            default: (!self.edit.monitors.is_empty()).then(|| "all".to_owned()),
        };
        let help = "With none selected, the bar shows on every monitor.";
        let mut edited = self.edit.monitors.clone();
        let reset = widgets::row(ui, c, "Monitors", Some(help), &state, |ui| {
            ui.spacing_mut().item_spacing.x = 14.0;
            for index in (0..count).rev() {
                let shown = edited.is_empty() || edited.contains(&index);
                let mut on = shown;
                widgets::checkbox(ui, c, &format!("Monitor {}", index + 1), &mut on);
                if on != shown {
                    edited = toggled(&edited, index, count);
                }
            }
        });
        self.edit.monitors = if reset { Vec::new() } else { edited };
    }

    fn family(&mut self, ui: &mut Ui, c: &Colors) {
        let mut fonts = self.fonts.clone();
        self.custom(ui, c, "font", |ui, value, changed| {
            let SettingValue::Text(name) = value else {
                return;
            };
            if fonts.is_empty() {
                widgets::text(ui, c, "Family", 240.0, name, "", changed);
                return;
            }
            if !name.is_empty() && !fonts.contains(name) {
                fonts.push(name.clone());
            }
            let options: Vec<_> = fonts.iter().map(|f| (f.clone(), f.as_str())).collect();
            widgets::dropdown(ui, c, "Family", 240.0, &options, name, changed);
        });
    }

    fn style_row(&mut self, ui: &mut Ui, c: &Colors) {
        let on =
            |page: &Self, key: &str| matches!(page.edit.get(key), Some(SettingValue::Bool(true)));
        let state = RowState {
            unsaved: STYLES
                .iter()
                .any(|(key, ..)| self.edit.get(key) != self.saved.get(key)),
            default: None,
        };
        let mut flipped = None;
        widgets::row(ui, c, "Style", None, &state, |ui| {
            Frame::new()
                .fill(c.control)
                .corner_radius(6)
                .inner_margin(Margin::same(2))
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.x = 2.0;
                    ui.horizontal(|ui| {
                        for (key, name, letter) in STYLES {
                            if style_button(ui, c, name, letter, on(self, key)).clicked() {
                                flipped = Some(key);
                            }
                        }
                    });
                });
        });
        if let Some(key) = flipped {
            self.edit.set(key, SettingValue::Bool(!on(self, key)));
        }
    }

    /// The six numbers of the pills, each over a label.
    fn grid(&mut self, ui: &mut Ui, c: &Colors) {
        Frame::new()
            .inner_margin(Margin {
                left: 16,
                right: 16,
                top: 12,
                bottom: 14,
            })
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 12.0;
                for line in GRID {
                    ui.columns(3, |columns| {
                        for (ui, key) in columns.iter_mut().zip(line) {
                            self.number(ui, c, key);
                        }
                    });
                }
            });
    }

    fn number(&mut self, ui: &mut Ui, c: &Colors, key: &str) {
        let (Some(descriptor), Some(saved), Some(SettingValue::Int(mut n))) = (
            schema::find_bar(key),
            self.saved.get(key),
            self.edit.get(key).cloned(),
        ) else {
            return;
        };
        let SettingKind::Int { min, max, unit } = descriptor.kind else {
            return;
        };
        let changed = SettingValue::Int(n) != *saved;
        ui.spacing_mut().item_spacing.y = 5.0;
        ui.vertical(|ui| {
            let label = eframe::egui::RichText::new(descriptor.label)
                .font(theme::regular(11.0))
                .color(c.dim_text);
            ui.label(label);
            widgets::number(
                ui,
                c,
                descriptor.label,
                &mut n,
                min..=max,
                0.2,
                unit,
                changed,
            );
        });
        self.edit.set(key, SettingValue::Int(n));
    }
}

/// `monitors` with `index` ticked or unticked, of `count`: all of them
/// is none, and the last one cannot be unticked.
fn toggled(monitors: &[usize], index: usize, count: usize) -> Vec<usize> {
    let mut now: Vec<usize> = if monitors.is_empty() {
        (0..count).collect()
    } else {
        monitors.to_vec()
    };
    match now.iter().position(|m| *m == index) {
        Some(at) if now.len() > 1 => {
            now.remove(at);
        }
        Some(_) => {}
        None => now.push(index),
    }
    now.sort_unstable();
    if now.len() == count { Vec::new() } else { now }
}

/// A square toggle showing `letter`, read out as `name`.
fn style_button(ui: &mut Ui, c: &Colors, name: &str, letter: &str, on: bool) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(28.0, 22.0), Sense::click());
    response.widget_info(|| WidgetInfo::selected(Role::CheckBox, true, on, name));
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        if on {
            painter.rect_filled(rect, 4.0, c.raised);
        } else if response.hovered() {
            painter.rect_filled(rect, 4.0, c.hover);
        }
        let (font, color) = if on {
            (theme::bold(13.0), c.text)
        } else {
            (theme::regular(13.0), c.dim_text)
        };
        painter.text(rect.center(), Align2::CENTER_CENTER, letter, font, color);
        if letter == "U" {
            underline(painter, rect, color);
        }
    }
    response
        .on_hover_text(name)
        .on_hover_cursor(CursorIcon::PointingHand)
}

fn underline(painter: &eframe::egui::Painter, rect: Rect, color: Color32) {
    let y = rect.center().y + 7.5;
    let half = 4.0;
    let line = [
        eframe::egui::pos2(rect.center().x - half, y),
        eframe::egui::pos2(rect.center().x + half, y),
    ];
    painter.line_segment(line, Stroke::new(1.0, color));
}
