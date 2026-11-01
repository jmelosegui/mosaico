//! The Animations page: how windows move into place, from `config.toml`.

use eframe::egui::Ui;
use mosaico_core::config::schema::SettingValue;
use mosaico_core::motion::Easing;

use super::{Entry, PageContext, Row, Section};
use crate::model::Settings;
use crate::widgets::{self, RowState};

pub const SECTIONS: &[Section] = &[
    (
        "Motion",
        &[
            Entry::Setting("animations.enabled"),
            Entry::Setting("animations.duration"),
            Entry::Drawn("animations.easing", easing),
        ],
    ),
    ("System", &[Entry::Setting("animations.follow_system")]),
];

const KEY: &str = "animations.easing";

/// The curve a custom easing starts from when it is picked.
const CUSTOM: &str = "cubic-bezier(0.2, 0, 0, 1)";

/// What the easing drop-down picks: a preset, or a curve of the user's
/// own, typed next to it.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Pick {
    Preset(Easing),
    Custom,
}

/// The easing row: a drop-down of the presets and "Custom curve", which
/// shows a field for the curve. A curve that does not parse says why
/// under the label, and Apply refuses it.
fn easing(ui: &mut Ui, cx: &PageContext, row: &Row, edit: &mut Settings) {
    let mut text = text(edit.get(KEY));
    let (saved, default) = (text_of(cx.saved), text_of(cx.defaults));
    let parsed = text.parse::<Easing>();
    let help = match &parsed {
        Err(e) => e.to_string(),
        Ok(_) => row.help().unwrap_or_default().to_owned(),
    };
    let st = RowState {
        unsaved: text != saved,
        default: (text != default).then(|| label_of(&default)),
    };
    let now = match parsed {
        Ok(Easing::CubicBezier(_)) | Err(_) => Pick::Custom,
        Ok(preset) => Pick::Preset(preset),
    };
    let mut options: Vec<_> = Easing::PRESETS
        .iter()
        .map(|e| (Pick::Preset(*e), e.label()))
        .collect();
    options.push((Pick::Custom, "Custom curve"));
    let mut pick = now;
    let reset = widgets::row(ui, cx.c, row.label, Some(&help), &st, |ui| {
        widgets::dropdown(ui, cx.c, row.label, 170.0, &options, &mut pick, st.unsaved);
        if pick == Pick::Custom && now == Pick::Custom {
            widgets::text(
                ui,
                cx.c,
                "Custom curve",
                190.0,
                &mut text,
                CUSTOM,
                st.unsaved,
            );
        }
    });
    text = match (reset, pick) {
        (true, _) => default,
        (false, Pick::Preset(preset)) => preset.to_string(),
        (false, Pick::Custom) if now != Pick::Custom => CUSTOM.into(),
        (false, Pick::Custom) => text,
    };
    edit.set(KEY, SettingValue::Text(text));
}

fn text(value: &SettingValue) -> String {
    match value {
        SettingValue::Text(s) => s.clone(),
        other => other.to_string(),
    }
}

fn text_of(settings: &Settings) -> String {
    text(settings.get(KEY))
}

/// How the row's default shows an easing.
fn label_of(text: &str) -> String {
    text.parse::<Easing>()
        .map_or_else(|_| text.to_owned(), |e| e.label().to_owned())
}
