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

/// Choices whose labels are too long for a segmented control.
const DROPDOWNS: [&str; 1] = ["layout.hiding"];

/// Numbers with a small range that read best on a track.
const SLIDERS: [&str; 5] = [
    "layout.gap",
    "layout.ratio",
    "borders.width",
    "animations.duration",
    "background_opacity",
];

/// Where the track of a slider ends, for settings whose largest value is
/// far beyond what is usually picked. Larger values can still be typed.
fn track_max<N: From<u8>>(key: &str) -> Option<N> {
    match key {
        "layout.gap" => Some(N::from(48)),
        "borders.width" => Some(N::from(16)),
        _ => None,
    }
}

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
        default: (value != default).then(|| shown(setting, theme, default)),
    };
    widgets::row(ui, c, setting.label, help, &st, |ui| {
        control(ui, c, setting, theme, value, st.unsaved);
    })
}

/// `value` as a row shows it in its default; an empty color is the one
/// `theme` has for the setting.
pub fn shown(setting: &SettingDescriptor, theme: Theme, value: &SettingValue) -> String {
    match (setting.kind, value) {
        (SettingKind::Int { unit, .. }, SettingValue::Int(n)) if !unit.is_empty() => {
            format!("{n} {unit}")
        }
        (SettingKind::Choice(options), SettingValue::Text(s)) => options
            .iter()
            .find(|(name, _)| name == s)
            .map_or_else(|| s.clone(), |(_, label)| (*label).to_owned()),
        (SettingKind::Color, SettingValue::Text(s)) if s.is_empty() => {
            match theme.default_color(setting.key) {
                Some(hex) => theme.color_name(&hex).map_or(hex.clone(), str::to_owned),
                None => "the theme's".into(),
            }
        }
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
            if SLIDERS.contains(&setting.key) {
                let track = min..=track_max(setting.key).unwrap_or(max);
                widgets::slider(ui, c, label, n, min..=max, track, 0.2, unit, changed);
            } else {
                widgets::number(ui, c, label, n, min..=max, 0.2, unit, changed);
            }
        }
        (SettingKind::Float { min, max, step }, SettingValue::Float(x)) => {
            let speed = step / 5.0;
            if SLIDERS.contains(&setting.key) {
                let track = min..=track_max(setting.key).unwrap_or(max);
                widgets::slider(ui, c, label, x, min..=max, track, speed, "", changed);
            } else {
                widgets::number(ui, c, label, x, min..=max, speed, "", changed);
            }
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
            if options.len() <= MAX_SEGMENTS && !DROPDOWNS.contains(&setting.key) {
                widgets::segmented(ui, c, &options, &mut chosen, changed);
            } else {
                let longest = options.iter().map(|(_, label)| label.len()).max();
                let width = (longest.unwrap_or(0) as f32 * 7.0 + 40.0).max(120.0);
                widgets::dropdown(ui, c, label, width, &options, &mut chosen, changed);
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
