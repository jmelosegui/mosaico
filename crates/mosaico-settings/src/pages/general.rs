//! The General page: theme, startup and logging, from `config.toml`.

use eframe::egui::Ui;
use mosaico_core::config::Theme;

use super::{PageContext, Row, Section, sections};
use crate::model::{General, LOG_LEVELS, LOG_SIZES_MB};
use crate::widgets::{self, RowState};

/// The file the page edits.
pub const FILE: &str = "config.toml";

const SECTIONS: &[Section<General>] = &[
    (
        "Appearance",
        &[
            (
                Row::new(
                    "Theme",
                    "Colors for window borders, the bar and this window.",
                ),
                theme,
            ),
            (Row::new("Flavor", ""), flavor),
        ],
    ),
    (
        "Startup",
        &[(
            Row::new("Start at sign-in", "Start the daemon when you sign in."),
            sign_in,
        )],
    ),
    (
        "Logging",
        &[
            (
                Row::new("Write a log file", "Logs go to the mosaico logs folder."),
                log_to_file,
            ),
            (
                Row::new("Level", "The least severe messages that are written."),
                log_level,
            ),
            (
                Row::new(
                    "Maximum file size",
                    "The log is rotated when it reaches this size.",
                ),
                log_size,
            ),
        ],
    ),
];

/// Shows the rows matching `query`, in their sections.
pub fn show(ui: &mut Ui, cx: &PageContext, query: &str, edit: &mut General) {
    sections(ui, cx, query, SECTIONS, edit);
}

/// What a row shows for a setting that is `now`, was `saved` and is
/// `default` on a new install.
fn state<T: PartialEq>(now: T, saved: T, default: T, text: impl Fn(&T) -> String) -> RowState {
    RowState {
        unsaved: now != saved,
        default: (now != default).then(|| text(&default)),
    }
}

fn on_off(on: &bool) -> String {
    if *on { "on" } else { "off" }.into()
}

fn theme(ui: &mut Ui, cx: &PageContext, row: &Row, edit: &mut General) {
    let st = state(
        edit.theme.name(),
        cx.saved.theme.name(),
        cx.defaults.theme.name(),
        |_| cx.defaults.theme.label().into(),
    );
    let options: Vec<_> = Theme::families().map(|t| (t.name(), t.label())).collect();
    let mut family = edit.theme.name();
    let reset = widgets::row(ui, cx.c, row.label, row.help(), &st, |ui| {
        widgets::dropdown(
            ui,
            cx.c,
            row.label,
            150.0,
            &options,
            &mut family,
            st.unsaved,
        );
    });
    if reset {
        edit.theme = cx.defaults.theme;
    } else if family != edit.theme.name() {
        let first = Theme::families().find(|t| t.name() == family);
        edit.theme = first.unwrap_or(edit.theme);
    }
}

fn flavor(ui: &mut Ui, cx: &PageContext, row: &Row, edit: &mut General) {
    let first = edit.theme.flavors().next().unwrap_or(edit.theme);
    let saved = (cx.saved.theme.name() == edit.theme.name()).then_some(cx.saved.theme);
    let st = RowState {
        unsaved: saved.is_some_and(|t| t != edit.theme),
        default: (edit.theme != first).then(|| first.flavor_label().into()),
    };
    let options: Vec<_> = edit
        .theme
        .flavors()
        .map(|t| (t, t.flavor_label()))
        .collect();
    let mut chosen = edit.theme;
    let reset = widgets::row(ui, cx.c, row.label, row.help(), &st, |ui| {
        widgets::segmented(ui, cx.c, &options, &mut chosen, st.unsaved);
    });
    edit.theme = if reset { first } else { chosen };
}

fn sign_in(ui: &mut Ui, cx: &PageContext, row: &Row, edit: &mut General) {
    let default = cx.defaults.start_at_sign_in;
    let st = state(
        edit.start_at_sign_in,
        cx.saved.start_at_sign_in,
        default,
        on_off,
    );
    let mut on = edit.start_at_sign_in;
    let reset = widgets::row(ui, cx.c, row.label, row.help(), &st, |ui| {
        widgets::toggle(ui, cx.c, row.label, &mut on);
    });
    edit.start_at_sign_in = if reset { default } else { on };
}

fn log_to_file(ui: &mut Ui, cx: &PageContext, row: &Row, edit: &mut General) {
    let st = state(
        edit.log_to_file,
        cx.saved.log_to_file,
        cx.defaults.log_to_file,
        on_off,
    );
    let help = format!("Logs go to {}.", cx.log_file);
    let mut on = edit.log_to_file;
    let reset = widgets::row(ui, cx.c, row.label, Some(&help), &st, |ui| {
        widgets::toggle(ui, cx.c, row.label, &mut on);
    });
    edit.log_to_file = if reset { cx.defaults.log_to_file } else { on };
}

fn log_level(ui: &mut Ui, cx: &PageContext, row: &Row, edit: &mut General) {
    let (now, saved, default) = (&edit.log_level, &cx.saved.log_level, &cx.defaults.log_level);
    let st = state(now, saved, default, |d| d.to_string());
    let options: Vec<_> = LOG_LEVELS.iter().map(|l| (l.to_string(), *l)).collect();
    let mut level = edit.log_level.clone();
    let reset = widgets::row(ui, cx.c, row.label, row.help(), &st, |ui| {
        widgets::dropdown(ui, cx.c, row.label, 100.0, &options, &mut level, st.unsaved);
    });
    edit.log_level = if reset { default.clone() } else { level };
}

fn log_size(ui: &mut Ui, cx: &PageContext, row: &Row, edit: &mut General) {
    let (saved, default) = (cx.saved.max_log_mb, cx.defaults.max_log_mb);
    let st = state(edit.max_log_mb, saved, default, |d| format!("{d} MB"));
    let mut mb = edit.max_log_mb;
    let reset = widgets::row(ui, cx.c, row.label, row.help(), &st, |ui| {
        widgets::number(ui, cx.c, &mut mb, LOG_SIZES_MB, "MB", st.unsaved);
    });
    edit.max_log_mb = if reset { default } else { mb };
}
