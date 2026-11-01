//! The Animations page: how windows move into place, from `config.toml`.

mod curve;
mod play;

use eframe::egui::Ui;
use mosaico_core::config::schema::SettingValue;
use mosaico_core::motion::{CubicBezier, Easing};

use super::{Entry, PageContext, Row, Section};
use crate::model::Settings;
use crate::widgets::{self, RowState};

pub const SECTIONS: &[Section] = &[
    (
        "Transitions",
        &[
            Entry::Setting("animations.enabled"),
            Entry::Setting("animations.duration"),
            Entry::Drawn("animations.easing", easing),
        ],
    ),
    ("System", &[Entry::Setting("animations.follow_system")]),
];

const KEY: &str = "animations.easing";

/// How long a transition takes, which Play shows the curve over.
const DURATION: &str = "animations.duration";

/// The curve a custom easing starts from when it is picked.
const CUSTOM: [f64; 4] = [0.2, 0.0, 0.0, 1.0];

/// What the easing drop-down picks: a preset, or a curve of the user's
/// own, whose points are edited under it.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Pick {
    Preset(Easing),
    Custom,
}

/// The easing row: a drop-down of the presets and "Custom", over a
/// picture of the curve and the four points of it, which can be edited
/// when the curve is a custom one. A curve that does not parse says why
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
    let shape = parsed.as_ref().ok().copied();
    let now = match shape {
        Some(Easing::CubicBezier(_)) | None => Pick::Custom,
        Some(preset) => Pick::Preset(preset),
    };
    let mut options: Vec<_> = Easing::PRESETS
        .iter()
        .map(|e| (Pick::Preset(*e), e.label()))
        .collect();
    options.push((Pick::Custom, "Custom"));
    let mut pick = now;
    let reset = widgets::row(ui, cx.c, row.label, Some(&help), &st, |ui| {
        widgets::dropdown(ui, cx.c, row.label, 190.0, &options, &mut pick, st.unsaved);
    });
    let mut points = shape.map_or(CUSTOM, Easing::control_points);
    let seconds = match edit.get(DURATION) {
        SettingValue::Int(ms) => *ms as f64 / 1000.0,
        _ => 0.0,
    };
    let custom = now == Pick::Custom;
    let edited = curve::show(ui, cx.c, shape, &mut points, custom, seconds);
    text = match (reset, pick) {
        (true, _) => default,
        (false, Pick::Preset(preset)) => preset.to_string(),
        (false, Pick::Custom) if now != Pick::Custom => bezier(CUSTOM).unwrap_or(text),
        (false, Pick::Custom) if edited => bezier(points).unwrap_or(text),
        (false, Pick::Custom) => text,
    };
    edit.set(KEY, SettingValue::Text(text));
}

/// The easing text of a curve with `points`, if they are a curve.
fn bezier([x1, y1, x2, y2]: [f64; 4]) -> Option<String> {
    let round = |v: f64| (v * 100.0).round() / 100.0;
    CubicBezier::new(round(x1), round(y1), round(x2), round(y2))
        .map(|curve| Easing::CubicBezier(curve).to_string())
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
