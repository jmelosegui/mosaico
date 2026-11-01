//! The pages of the settings window, and what they share.

pub mod general;

use eframe::egui::{RichText, Ui};

use crate::icons::Icon;
use crate::model::General;
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
        self == Self::General
    }
}

/// What every row of a page reads.
pub struct PageContext<'a> {
    pub c: &'a Colors,
    /// The settings as they were loaded.
    pub saved: &'a General,
    /// The settings of a new install.
    pub defaults: &'a General,
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

/// Draws one row, editing the page's settings.
pub type Draw<S> = fn(&mut Ui, &PageContext, &Row, &mut S);

/// A titled group of rows, each with how it is drawn.
pub type Section<S> = (&'static str, &'static [(Row, Draw<S>)]);

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

/// Shows the rows of `sections` that match `query`, leaving out
/// sections with none.
fn sections<S>(
    ui: &mut Ui,
    cx: &PageContext,
    query: &str,
    sections: &[Section<S>],
    settings: &mut S,
) {
    let mut any = false;
    for (title, rows) in sections {
        let shown: Vec<_> = rows.iter().filter(|(row, _)| row.matches(query)).collect();
        if shown.is_empty() {
            continue;
        }
        any = true;
        widgets::section(ui, cx.c, title, |ui| {
            for (i, (row, draw)) in shown.iter().enumerate() {
                if i > 0 {
                    widgets::divider(ui, cx.c);
                }
                draw(ui, cx, row, settings);
            }
        });
    }
    if !any {
        let text = format!("No settings match \u{201c}{}\u{201d}.", query.trim());
        ui.label(RichText::new(text).color(cx.c.dim_text));
    }
}
