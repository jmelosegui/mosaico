//! The kinds of widget a bar shows, named as `bar.toml` names them.

use std::fmt;
use std::str::FromStr;

use super::bar::WidgetConfig;

/// A kind of bar widget: the `type` of a `[[left]]` or `[[right]]` entry,
/// without its settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WidgetKind {
    /// The workspace numbers.
    Workspaces,
    /// The layout name and the monocle mark.
    Layout,
    /// The time.
    Clock,
    /// The date.
    Date,
    /// The memory in use.
    Ram,
    /// The processor use.
    Cpu,
    /// The notice of a newer version.
    Update,
    /// The icon of the focused window.
    ActiveWindow,
    /// What is playing.
    Media,
    /// The mark shown while the hotkeys are paused.
    Paused,
}

impl WidgetKind {
    /// Every kind, in the order the docs list them.
    pub const ALL: [Self; 10] = [
        Self::Workspaces,
        Self::ActiveWindow,
        Self::Layout,
        Self::Clock,
        Self::Date,
        Self::Ram,
        Self::Cpu,
        Self::Update,
        Self::Media,
        Self::Paused,
    ];

    /// The `type` that names the kind in `bar.toml`, such as
    /// `"active_window"`.
    pub fn name(self) -> &'static str {
        match self {
            Self::Workspaces => "workspaces",
            Self::Layout => "layout",
            Self::Clock => "clock",
            Self::Date => "date",
            Self::Ram => "ram",
            Self::Cpu => "cpu",
            Self::Update => "update",
            Self::ActiveWindow => "active_window",
            Self::Media => "media",
            Self::Paused => "paused",
        }
    }
}

impl FromStr for WidgetKind {
    type Err = String;

    /// The kind named `s`, as a widget's `type` names it.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|kind| kind.name() == s)
            .ok_or_else(|| format!("unknown widget: {s}"))
    }
}

impl fmt::Display for WidgetKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl WidgetConfig {
    /// The kind of the widget.
    pub fn kind(&self) -> WidgetKind {
        match self {
            Self::Workspaces { .. } => WidgetKind::Workspaces,
            Self::Layout { .. } => WidgetKind::Layout,
            Self::Clock { .. } => WidgetKind::Clock,
            Self::Date { .. } => WidgetKind::Date,
            Self::Ram { .. } => WidgetKind::Ram,
            Self::Cpu { .. } => WidgetKind::Cpu,
            Self::Update { .. } => WidgetKind::Update,
            Self::ActiveWindow { .. } => WidgetKind::ActiveWindow,
            Self::Media { .. } => WidgetKind::Media,
            Self::Paused { .. } => WidgetKind::Paused,
        }
    }
}

#[cfg(test)]
#[path = "widget_kind_tests.rs"]
mod tests;
