//! The Bar page: how the status bar looks and what it shows, from
//! `bar.toml`, in the tabs Appearance, Widgets and Mouse.

mod appearance;
mod clicks;
mod columns;
mod look;
mod pills;
mod preview;
mod state;

use eframe::egui::Ui;
use mosaico_core::config::schema::{self, SettingDescriptor, SettingValue};
use mosaico_core::config::{ConfigStore, Theme};

use super::{FilePage, Page, Row, setting};
use crate::theme::Colors;
use crate::widgets::{self, RowState};
use state::BarState;

/// A titled group of the keys of `bar.toml`, drawn from their
/// descriptors.
type Section = (&'static str, &'static [&'static str]);

/// The tabs, in order.
const TABS: [&str; 3] = ["Appearance", "Widgets", "Mouse"];

/// Rows the window shows without their help, which a search still reads.
const QUIET: [&str; 3] = ["enabled", "background_opacity", "font_size"];

/// Every single setting, for when a search looks across the tabs.
const SEARCH: &[Section] = &[
    ("Bar", &["enabled", "height", "background_opacity"]),
    (
        "Font",
        &[
            "font",
            "font_size",
            "font_bold",
            "font_italic",
            "font_underline",
        ],
    ),
    (
        "Pills and spacing",
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
    ("Colors", appearance::COLORS),
];

/// What a search finds the Widgets tab by, beside the names of the
/// widgets on the bar.
const WIDGETS: Row = Row::new(
    "Widgets",
    "Widgets on the left are drawn from the left edge, those on the right from the right edge.",
);

/// What a search finds the table of click actions by.
const CLICKS: Row = Row::new("Click actions", "A click on a widget");

/// The single settings of `keys` matching `query`.
fn found(keys: &[&str], query: &str) -> Vec<&'static SettingDescriptor> {
    keys.iter()
        .filter_map(|key| schema::find_bar(key))
        .filter(|s| Row::keyed(s.key, s.label, s.help).matches(query))
        .collect()
}

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
    tab: usize,
    /// How many monitors are connected.
    monitors: usize,
    /// The installed font families.
    fonts: Vec<String>,
    columns: columns::Editor,
}

impl BarPage {
    /// The page for the `bar.toml` in `store`, with no edits.
    pub fn new(store: &ConfigStore, monitors: usize, fonts: &[String]) -> Self {
        let defaults = BarState::defaults();
        let mut page = Self {
            store: store.clone(),
            saved: defaults.clone(),
            edit: defaults.clone(),
            defaults,
            problem: None,
            theme: Theme::default(),
            tab: 0,
            monitors,
            fonts: fonts.to_vec(),
            columns: columns::Editor::default(),
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

    fn sections(&mut self, ui: &mut Ui, c: &Colors, query: &str, sections: &[Section]) {
        for (title, keys) in sections {
            let shown = found(keys, query);
            if shown.is_empty() {
                continue;
            }
            widgets::section(ui, c, title, |ui| {
                for (i, setting) in shown.iter().enumerate() {
                    if i > 0 {
                        widgets::divider(ui, c);
                    }
                    self.setting(ui, c, setting.key);
                }
            });
        }
    }

    /// The row of the setting at `key`, from its descriptor.
    fn setting(&mut self, ui: &mut Ui, c: &Colors, key: &str) {
        let Some(descriptor) = schema::find_bar(key) else {
            return;
        };
        let (Some(saved), Some(default), Some(mut value)) = (
            self.saved.get(key),
            self.defaults.get(key),
            self.edit.get(key).cloned(),
        ) else {
            return;
        };
        let help = Row::new(descriptor.label, descriptor.help)
            .help()
            .filter(|_| !QUIET.contains(&key));
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

    /// The row of the setting at `key` with a control of the page's own.
    fn custom(
        &mut self,
        ui: &mut Ui,
        c: &Colors,
        key: &str,
        control: impl FnOnce(&mut Ui, &mut SettingValue, bool),
    ) {
        let Some(descriptor) = schema::find_bar(key) else {
            return;
        };
        let (Some(saved), Some(default), Some(mut value)) = (
            self.saved.get(key),
            self.defaults.get(key),
            self.edit.get(key).cloned(),
        ) else {
            return;
        };
        let state = RowState {
            unsaved: value != *saved,
            default: (value != *default).then(|| setting::shown(descriptor, self.theme, default)),
        };
        let help = Row::new(descriptor.label, descriptor.help).help();
        let reset = widgets::row(ui, c, descriptor.label, help, &state, |ui| {
            control(ui, &mut value, state.unsaved);
        });
        let value = if reset { default.clone() } else { value };
        self.edit.set(key, value);
    }

    /// Whether a search for `query` finds the Widgets tab: by its name or
    /// by a widget on the bar.
    fn widgets_match(&self, query: &str) -> bool {
        let widgets = &self.edit.widgets;
        let mut on_bar = widgets.left.iter().chain(&widgets.right);
        WIDGETS.matches(query)
            || on_bar.any(|w| Row::new(columns::label(w.kind), "").matches(query))
    }

    /// The rows matching `query` of every tab, for a search.
    fn search(&mut self, ui: &mut Ui, c: &Colors, query: &str) {
        self.sections(ui, c, query, SEARCH);
        if self.widgets_match(query) {
            self.widgets_tab(ui, c);
        }
        if CLICKS.matches(query) {
            self.click_table(ui, c);
        }
    }
}
impl FilePage for BarPage {
    fn page(&self) -> Page {
        Page::Bar
    }

    fn changes(&self) -> usize {
        self.edit.changes_from(&self.saved)
    }

    fn matches(&self, query: &str) -> usize {
        let rows: usize = SEARCH
            .iter()
            .map(|(_, keys)| found(keys, query).len())
            .sum();
        rows + usize::from(self.widgets_match(query)) + usize::from(CLICKS.matches(query))
    }

    fn problem(&self) -> Option<&str> {
        self.problem.as_deref()
    }

    fn show(&mut self, ui: &mut Ui, c: &Colors, query: &str) {
        if !query.trim().is_empty() {
            self.search(ui, c, query);
            return;
        }
        widgets::tabs(ui, c, &TABS, &mut self.tab);
        if self.tab < 2 {
            preview::show(ui, c, &self.edit, self.theme);
        }
        match self.tab {
            0 => self.appearance(ui, c),
            1 => self.widgets_tab(ui, c),
            _ => self.mouse(ui, c),
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
