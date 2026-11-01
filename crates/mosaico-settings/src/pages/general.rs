//! The General page: theme, startup and logging, from `config.toml`.

use eframe::egui::Ui;
use mosaico_core::config::Theme;

use super::{Entry, PageContext, Row, Section, setting};
use crate::model::Settings;
use crate::widgets::{self, RowState};

pub const SECTIONS: &[Section] = &[
    (
        "Appearance",
        &[
            Entry::Drawn("theme.name", theme),
            Entry::Drawn("theme.flavor", flavor),
        ],
    ),
    (
        "Startup",
        &[
            Entry::Setting("tray.enabled"),
            Entry::Custom(
                Row::new("Start at sign-in", "Start the daemon when you sign in."),
                sign_in,
            ),
        ],
    ),
    (
        "Logging",
        &[
            Entry::Drawn("logging.enabled", log_to_file),
            Entry::Setting("logging.level"),
            Entry::Setting("logging.max_file_mb"),
        ],
    ),
];

fn theme(ui: &mut Ui, cx: &PageContext, row: &Row, edit: &mut Settings) {
    let (now, saved, default) = (edit.theme(), cx.saved.theme(), cx.defaults.theme());
    let st = RowState {
        unsaved: now.name() != saved.name(),
        default: (now.name() != default.name()).then(|| default.label().into()),
    };
    let options: Vec<_> = Theme::families().map(|t| (t.name(), t.label())).collect();
    let mut family = now.name();
    let reset = widgets::row(ui, cx.c, row.label, row.help(), &st, |ui| {
        widgets::dropdown(
            ui,
            cx.c,
            row.label,
            168.0,
            &options,
            &mut family,
            st.unsaved,
        );
    });
    if reset {
        edit.set_theme(default);
    } else if family != now.name() {
        let first = Theme::families().find(|t| t.name() == family);
        edit.set_theme(first.unwrap_or(now));
    }
}

fn flavor(ui: &mut Ui, cx: &PageContext, row: &Row, edit: &mut Settings) {
    let now = edit.theme();
    let first = now.flavors().next().unwrap_or(now);
    let saved = cx.saved.theme();
    let st = RowState {
        unsaved: saved.name() == now.name() && saved != now,
        default: (now != first).then(|| first.flavor_label().into()),
    };
    let options: Vec<_> = now.flavors().map(|t| (t, t.flavor_label())).collect();
    let mut chosen = now;
    let reset = widgets::row(ui, cx.c, row.label, row.help(), &st, |ui| {
        widgets::segmented(ui, cx.c, &options, &mut chosen, st.unsaved);
    });
    edit.set_theme(if reset { first } else { chosen });
}

fn sign_in(ui: &mut Ui, cx: &PageContext, row: &Row, edit: &mut Settings) {
    let default = cx.defaults.start_at_sign_in;
    let now = edit.start_at_sign_in;
    let st = RowState {
        unsaved: now != cx.saved.start_at_sign_in,
        default: (now != default).then(|| if default { "on" } else { "off" }.into()),
    };
    let mut on = now;
    let reset = widgets::row(ui, cx.c, row.label, row.help(), &st, |ui| {
        widgets::toggle(ui, cx.c, row.label, &mut on);
    });
    edit.start_at_sign_in = if reset { default } else { on };
}

fn log_to_file(ui: &mut Ui, cx: &PageContext, _row: &Row, edit: &mut Settings) {
    let help = format!("Logs go to {}.", cx.log_file);
    setting::draw(ui, cx, "logging.enabled", Some(&help), edit);
}
