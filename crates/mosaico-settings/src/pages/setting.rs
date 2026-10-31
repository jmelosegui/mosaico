//! A row for any setting of `config.toml`, with the control its
//! descriptor calls for.

use eframe::egui::Ui;
use mosaico_core::config::Theme;
use mosaico_core::config::schema::{self, SettingDescriptor, SettingKind, SettingValue};

use super::PageContext;
use crate::model::Settings;
use crate::theme::Colors;
use crate::widgets::{self, RowState};

/// Choices with more options than this get a drop-down rather than a
/// segmented control.
const MAX_SEGMENTS: usize = 3;

/// Numbers with a small range that read best on a track.
const SLIDERS: [&str; 5] = [
    "layout.gap",
    "layout.ratio",
    "borders.width",
    "animations.duration",
    "background_opacity",
];

/// Draws the row of the setting at `key`, with `help` under its label.
pub fn draw(ui: &mut Ui, cx: &PageContext, key: &str, help: Option<&str>, edit: &mut Settings) {
    let setting = schema::find(key).unwrap_or_else(|| panic!("no setting {key}"));
    let default = cx.defaults.get(key);
    let mut value = edit.get(key).clone();
    let reset = draw_value(
        ui,
        cx.c,
        edit.theme(),
        setting,
        help,
        (cx.saved.get(key), default),
        &mut value,
    );
    edit.set(key, if reset { default.clone() } else { value });
}

/// Draws the row of `setting`, editing `value`, which is `saved` in the
/// file and `default` when the file leaves it out. Colors offer the
/// palette of `theme`. Returns whether the reset button was clicked,
/// which leaves `value` for the caller to set to the default.
pub fn draw_value(
    ui: &mut Ui,
    c: &Colors,
    theme: Theme,
    setting: &SettingDescriptor,
    help: Option<&str>,
    (saved, default): (&SettingValue, &SettingValue),
    value: &mut SettingValue,
) -> bool {
    let st = RowState {
        unsaved: value != saved,
        default: (value != default).then(|| shown(setting, default)),
    };
    widgets::row(ui, c, setting.label, help, &st, |ui| {
        control(ui, c, setting, theme, value, st.unsaved);
    })
}

/// `value` as a row shows it in its default.
pub fn shown(setting: &SettingDescriptor, value: &SettingValue) -> String {
    match (setting.kind, value) {
        (SettingKind::Int { unit, .. }, SettingValue::Int(n)) if !unit.is_empty() => {
            format!("{n} {unit}")
        }
        (SettingKind::Choice(options), SettingValue::Text(s)) => options
            .iter()
            .find(|(name, _)| name == s)
            .map_or_else(|| s.clone(), |(_, label)| (*label).to_owned()),
        (SettingKind::Color, SettingValue::Text(s)) if s.is_empty() => "the theme's".into(),
        _ => value.to_string(),
    }
}

fn control(
    ui: &mut Ui,
    c: &Colors,
    setting: &SettingDescriptor,
    theme: Theme,
    value: &mut SettingValue,
    changed: bool,
) {
    let label = setting.label;
    match (setting.kind, value) {
        (SettingKind::Bool, SettingValue::Bool(on)) => {
            widgets::toggle(ui, c, label, on);
        }
        (SettingKind::Int { min, max, unit }, SettingValue::Int(n)) => {
            let draw = if SLIDERS.contains(&setting.key) {
                widgets::slider
            } else {
                widgets::number
            };
            draw(ui, c, label, n, min..=max, 0.2, unit, changed);
        }
        (SettingKind::Float { min, max, step }, SettingValue::Float(x)) => {
            let draw = if SLIDERS.contains(&setting.key) {
                widgets::slider
            } else {
                widgets::number
            };
            draw(ui, c, label, x, min..=max, step / 5.0, "", changed);
        }
        (SettingKind::Choice(options), SettingValue::Text(s)) => {
            let options: Vec<_> = options
                .iter()
                .map(|(name, label)| (*name, *label))
                .collect();
            let mut chosen = options
                .iter()
                .find(|(name, _)| name == s)
                .map_or(options[0].0, |(name, _)| *name);
            if options.len() <= MAX_SEGMENTS {
                widgets::segmented(ui, c, &options, &mut chosen, changed);
            } else {
                widgets::dropdown(ui, c, label, 140.0, &options, &mut chosen, changed);
            }
            *s = chosen.to_owned();
        }
        (SettingKind::Color, SettingValue::Text(s)) => {
            let fallback = theme.default_color(setting.key).unwrap_or_default();
            let picker = widgets::ColorPicker {
                label,
                theme,
                fallback: &fallback,
                allow_none: setting.allows_none(),
                changed,
            };
            widgets::color(ui, c, &picker, s);
        }
        (_, SettingValue::Text(s)) => {
            widgets::text(ui, c, label, 200.0, s, "", changed);
        }
        _ => {}
    }
}
