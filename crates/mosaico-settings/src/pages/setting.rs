//! A row for any setting of `config.toml`, with the control its
//! descriptor calls for.

use eframe::egui::Ui;
use mosaico_core::config::Theme;
use mosaico_core::config::schema::{self, SettingDescriptor, SettingKind, SettingValue};

use super::PageContext;
use crate::model::Settings;
use crate::widgets::{self, RowState};

/// Choices with more options than this get a drop-down rather than a
/// segmented control.
const MAX_SEGMENTS: usize = 3;

/// Draws the row of the setting at `key`, with `help` under its label.
pub fn draw(ui: &mut Ui, cx: &PageContext, key: &str, help: Option<&str>, edit: &mut Settings) {
    let setting = schema::find(key).unwrap_or_else(|| panic!("no setting {key}"));
    let default = cx.defaults.get(key);
    let mut value = edit.get(key).clone();
    let st = RowState {
        unsaved: &value != cx.saved.get(key),
        default: (&value != default).then(|| shown(setting, default)),
    };
    let reset = widgets::row(ui, cx.c, setting.label, help, &st, |ui| {
        control(ui, cx, setting, edit.theme(), &mut value, st.unsaved);
    });
    edit.set(key, if reset { default.clone() } else { value });
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
    cx: &PageContext,
    setting: &SettingDescriptor,
    theme: Theme,
    value: &mut SettingValue,
    changed: bool,
) {
    let (c, label) = (cx.c, setting.label);
    match (setting.kind, value) {
        (SettingKind::Bool, SettingValue::Bool(on)) => {
            widgets::toggle(ui, c, label, on);
        }
        (SettingKind::Int { min, max, unit }, SettingValue::Int(n)) => {
            widgets::number(ui, c, label, n, min..=max, 0.2, unit, changed);
        }
        (SettingKind::Float { min, max, step }, SettingValue::Float(x)) => {
            widgets::number(ui, c, label, x, min..=max, step / 5.0, "", changed);
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
            let picker = widgets::ColorPicker {
                label,
                theme,
                fallback: theme.border_default(setting.key).unwrap_or_default(),
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
