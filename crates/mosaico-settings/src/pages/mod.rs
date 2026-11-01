//! The pages of the settings window, and what they share.

mod animations;
mod borders;
mod file_page;
mod general;
mod keybindings;
mod layout;
mod mouse;
mod setting;
mod workspaces;

pub use file_page::{FilePage, FilePages};

use eframe::egui::{Frame, Margin, RichText, Stroke, Ui};
use mosaico_core::config::ConfigStore;

use crate::icons::Icon;
use mosaico_core::config::schema;

use crate::model::Settings;
use crate::theme::{self, Colors};
use crate::widgets;

/// A page of the navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    General,
    Layout,
    Workspaces,
    Borders,
    Mouse,
    Animations,
    Bar,
    Keybindings,
    Rules,
}

impl Page {
    /// Every page, in the order of the navigation.
    pub const ALL: [Page; 9] = [
        Self::General,
        Self::Layout,
        Self::Workspaces,
        Self::Borders,
        Self::Mouse,
        Self::Animations,
        Self::Bar,
        Self::Keybindings,
        Self::Rules,
    ];

    /// The page's name in the navigation and its header.
    pub fn label(self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Layout => "Layout",
            Self::Workspaces => "Workspaces",
            Self::Borders => "Borders",
            Self::Mouse => "Mouse",
            Self::Animations => "Animations",
            Self::Bar => "Bar",
            Self::Keybindings => "Keybindings",
            Self::Rules => "Rules",
        }
    }

    /// The page's icon in the navigation.
    pub fn icon(self) -> Icon {
        match self {
            Self::General => Icon::General,
            Self::Layout => Icon::Layout,
            Self::Workspaces => Icon::Workspaces,
            Self::Borders => Icon::Borders,
            Self::Mouse => Icon::Mouse,
            Self::Animations => Icon::Animations,
            Self::Bar => Icon::Bar,
            Self::Keybindings => Icon::Keybindings,
            Self::Rules => Icon::Rules,
        }
    }

    /// Whether the page can be opened yet.
    pub fn is_ready(self) -> bool {
        self.sections().is_some() || FILE_PAGES.contains(&self)
    }

    /// The file the page edits.
    pub fn file(self) -> &'static str {
        match self {
            Self::Bar => "bar.toml",
            Self::Keybindings => "keybindings.toml",
            Self::Rules => "user-rules.toml",
            _ => "config.toml",
        }
    }

    /// The page's rows of `config.toml` settings, if it has them.
    fn sections(self) -> Option<&'static [Section]> {
        match self {
            Self::General => Some(general::SECTIONS),
            Self::Layout => Some(layout::SECTIONS),
            Self::Mouse => Some(mouse::SECTIONS),
            Self::Animations => Some(animations::SECTIONS),
            Self::Workspaces => Some(workspaces::SECTIONS),
            Self::Borders => Some(borders::SECTIONS),
            _ => None,
        }
    }
}

/// The pages that edit a file of their own; `file_pages` makes one of
/// each.
const FILE_PAGES: &[Page] = &[Page::Keybindings];

/// The pages that edit a file of their own, reading their files from
/// `store`.
pub fn file_pages(store: &ConfigStore) -> FilePages {
    FilePages::new(vec![Box::new(keybindings::KeybindingsPage::new(
        store.clone(),
    ))])
}

/// Shows the rows of `page` matching `query`, in their sections.
pub fn show(ui: &mut Ui, cx: &PageContext, page: Page, query: &str, edit: &mut Settings) {
    if let Some(list) = page.sections() {
        sections(ui, cx, query, list, edit);
    }
}

/// What every row of a page reads.
pub struct PageContext<'a> {
    pub c: &'a Colors,
    /// The settings as they were loaded.
    pub saved: &'a Settings,
    /// The settings of a new install.
    pub defaults: &'a Settings,
    /// Where the log file is written, as shown to the user.
    pub log_file: &'a str,
}

/// A setting's label and help, which search matches.
pub struct Row {
    pub label: &'static str,
    help: &'static str,
}

impl Row {
    pub const fn new(label: &'static str, help: &'static str) -> Self {
        Self { label, help }
    }

    /// The help under the label, if the row has any.
    pub fn help(&self) -> Option<&'static str> {
        (!self.help.is_empty()).then_some(self.help)
    }

    /// Whether the label or the help contains `query`, ignoring case.
    pub fn matches(&self, query: &str) -> bool {
        let query = query.trim().to_lowercase();
        [self.label, self.help]
            .iter()
            .any(|text| text.to_lowercase().contains(&query))
    }
}

/// Draws one row, editing the settings.
pub type Draw = fn(&mut Ui, &PageContext, &Row, &mut Settings);

/// A row of a page.
pub enum Entry {
    /// A setting of `config.toml`, drawn from its descriptor: a switch, a
    /// number, a choice or a line of text, by the values it takes.
    Setting(&'static str),
    /// A setting of `config.toml`, labelled from its descriptor and drawn
    /// by the page.
    Drawn(&'static str, Draw),
    /// A row that is not a setting of a file, drawn by the page.
    Custom(Row, Draw),
}

impl Entry {
    /// The row's label and help.
    fn row(&self) -> Row {
        match self {
            Self::Setting(key) | Self::Drawn(key, _) => {
                let setting = schema::find(key).unwrap_or_else(|| panic!("no setting {key}"));
                Row::new(setting.label, setting.help)
            }
            Self::Custom(row, _) => Row::new(row.label, row.help),
        }
    }

    fn draw(&self, ui: &mut Ui, cx: &PageContext, row: &Row, settings: &mut Settings) {
        match self {
            Self::Setting(key) => setting::draw(ui, cx, key, row.help(), settings),
            Self::Drawn(_, draw) | Self::Custom(_, draw) => draw(ui, cx, row, settings),
        }
    }
}

/// A titled group of rows.
pub type Section = (&'static str, &'static [Entry]);

/// A page's name over the file it edits.
pub fn header(ui: &mut Ui, c: &Colors, page: Page, file: &str) {
    ui.label(RichText::new(page.label()).font(theme::bold(18.0)));
    ui.label(
        RichText::new(file)
            .font(theme::regular(11.0))
            .color(c.faint_text),
    );
    ui.add_space(16.0);
}

/// Why `file` cannot be edited: what is wrong with it, over the page.
pub fn problem(ui: &mut Ui, c: &Colors, file: &str, problem: &str) {
    Frame::new()
        .fill(c.card)
        .stroke(Stroke::new(1.0, c.warning))
        .corner_radius(8)
        .inner_margin(Margin::symmetric(16, 12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            let title = format!("{file} has a problem, so it can't be edited here");
            ui.label(RichText::new(title).font(theme::bold(13.0)));
            ui.add_space(4.0);
            ui.label(
                RichText::new(problem)
                    .font(theme::regular(12.0))
                    .color(c.dim_text),
            );
            ui.add_space(4.0);
            ui.label(
                RichText::new("Fix it in an editor; this window picks the fix up.")
                    .font(theme::regular(11.0))
                    .color(c.faint_text),
            );
        });
    ui.add_space(16.0);
}

/// Shows the rows of `sections` that match `query`, leaving out
/// sections with none.
fn sections(
    ui: &mut Ui,
    cx: &PageContext,
    query: &str,
    sections: &[Section],
    settings: &mut Settings,
) {
    let mut any = false;
    for (title, entries) in sections {
        let shown: Vec<_> = entries
            .iter()
            .map(|entry| (entry, entry.row()))
            .filter(|(_, row)| row.matches(query))
            .collect();
        if shown.is_empty() {
            continue;
        }
        any = true;
        widgets::section(ui, cx.c, title, |ui| {
            for (i, (entry, row)) in shown.iter().enumerate() {
                if i > 0 {
                    widgets::divider(ui, cx.c);
                }
                entry.draw(ui, cx, row, settings);
            }
        });
    }
    if !any {
        let text = format!("No settings match \u{201c}{}\u{201d}.", query.trim());
        ui.label(RichText::new(text).color(cx.c.dim_text));
    }
}
