//! The Bar page: how the status bar looks and what it shows, from
//! `bar.toml`.

mod order;
mod state;

use eframe::egui::Ui;
use mosaico_core::config::schema::{self, SettingDescriptor};
use mosaico_core::config::{ConfigStore, Theme};

use super::{FilePage, Page, Row, setting};
use crate::theme::Colors;
use crate::widgets;
use state::BarState;

/// A titled group of the keys of `bar.toml`, drawn from their
/// descriptors.
type Section = (&'static str, &'static [&'static str]);

/// The sections before the widget order.
const BEFORE: &[Section] = &[(
    "Bar",
    &[
        "enabled",
        "height",
        "background_opacity",
        "font",
        "font_size",
    ],
)];

/// The sections after it.
const AFTER: &[Section] = &[
    ("Text", &["font_bold", "font_italic", "font_underline"]),
    (
        "Pills",
        &[
            "padding",
            "pill_padding",
            "pill_radius",
            "pill_border_width",
            "item_gap",
            "workspace_gap",
            "separator",
        ],
    ),
    ("Mouse", &["mouse.enabled", "mouse.scroll"]),
    (
        "Colors",
        &[
            "colors.background",
            "colors.foreground",
            "colors.active_workspace",
            "colors.active_workspace_text",
            "colors.inactive_workspace",
            "colors.separator",
            "colors.accent",
            "colors.widget_background",
            "colors.pill_border",
        ],
    ),
];

/// The page editing `bar.toml`.
pub struct BarPage {
    store: ConfigStore,
    saved: BarState,
    edit: BarState,
    defaults: BarState,
    problem: Option<String>,
    /// The theme of the saved `config.toml`, whose palette the colors
    /// offer.
    theme: Theme,
}

impl BarPage {
    /// The page for the `bar.toml` in `store`, with no edits.
    pub fn new(store: &ConfigStore) -> Self {
        let defaults = BarState::defaults();
        let mut page = Self {
            store: store.clone(),
            saved: defaults.clone(),
            edit: defaults.clone(),
            defaults,
            problem: None,
            theme: Theme::default(),
        };
        page.load();
        page
    }

    /// Reads the file, dropping any edits. A file that cannot be edited
    /// leaves the defaults and the reason.
    fn load(&mut self) {
        let (state, problem) = match state::load(&self.store) {
            Ok(state) => (state, None),
            Err(problem) => (self.defaults.clone(), Some(problem)),
        };
        self.problem = problem;
        self.theme = self
            .store
            .try_load_config()
            .map(|config| config.theme.resolve())
            .unwrap_or_default();
        self.saved = state.clone();
        self.edit = state;
    }

    fn sections(&mut self, ui: &mut Ui, c: &Colors, query: &str, sections: &[Section]) -> bool {
        let mut any = false;
        for (title, keys) in sections {
            let shown: Vec<_> = keys
                .iter()
                .filter_map(|key| schema::find_bar(key))
                .filter(|s| Row::new(s.label, s.help).matches(query))
                .collect();
            if shown.is_empty() {
                continue;
            }
            any = true;
            widgets::section(ui, c, title, |ui| {
                for (i, setting) in shown.iter().enumerate() {
                    if i > 0 {
                        widgets::divider(ui, c);
                    }
                    self.setting(ui, c, setting);
                }
            });
        }
        any
    }

    fn setting(&mut self, ui: &mut Ui, c: &Colors, descriptor: &SettingDescriptor) {
        let key = descriptor.key;
        let (Some(saved), Some(default), Some(mut value)) = (
            self.saved.get(key),
            self.defaults.get(key),
            self.edit.get(key).cloned(),
        ) else {
            return;
        };
        let help = Row::new(descriptor.label, descriptor.help).help();
        let reset = setting::draw_value(
            ui,
            c,
            self.theme,
            descriptor,
            help,
            (saved, default),
            &mut value,
        );
        let value = if reset { default.clone() } else { value };
        self.edit.set(key, value);
    }
}

impl FilePage for BarPage {
    fn page(&self) -> Page {
        Page::Bar
    }

    fn changes(&self) -> usize {
        self.edit.changes_from(&self.saved)
    }

    fn problem(&self) -> Option<&str> {
        self.problem.as_deref()
    }

    fn show(&mut self, ui: &mut Ui, c: &Colors, query: &str) {
        let mut any = self.sections(ui, c, query, BEFORE);
        any |= order::show(ui, c, query, &self.saved.widgets, &mut self.edit.widgets);
        any |= self.sections(ui, c, query, AFTER);
        if !any {
            let text = format!("No settings match \u{201c}{}\u{201d}.", query.trim());
            ui.label(eframe::egui::RichText::new(text).color(c.dim_text));
        }
    }

    fn apply(&mut self) -> Result<(), String> {
        state::save(&self.store, &self.saved, &self.edit)?;
        self.load();
        Ok(())
    }

    fn revert(&mut self) {
        self.edit = self.saved.clone();
    }

    fn reload(&mut self) {
        self.load();
    }
}
