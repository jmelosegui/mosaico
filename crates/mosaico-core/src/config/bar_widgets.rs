//! Which widgets the bar shows, where, and in what order.

use super::bar::{BarConfig, WidgetConfig};
use super::widget_kind::WidgetKind;

/// A side of the bar the widgets are on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarZone {
    /// The left side, drawn from the left.
    Left,
    /// The right side, drawn from the right: the first widget is the
    /// rightmost.
    Right,
}

impl BarZone {
    /// Both sides, left first.
    pub const ALL: [Self; 2] = [Self::Left, Self::Right];

    /// The key of the side in `bar.toml`.
    pub fn name(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
        }
    }

    /// The other side.
    pub fn other(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

/// A widget of the bar with the settings the window edits: whether it is
/// shown and how it looks. What it shows beyond that, such as how long
/// the media text may be, stays as the file has it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BarWidget {
    /// What the widget shows.
    pub kind: WidgetKind,
    /// Whether the bar shows it.
    pub enabled: bool,
    /// The format of the time or date, for the widgets that show one.
    pub format: Option<String>,
    /// The icon text before the widget's own. Empty for none.
    pub icon: String,
    /// The text color, a `#rrggbb` value or a theme color name. Empty
    /// for the widget's own.
    pub color: String,
}

impl BarWidget {
    /// A widget of `kind` with the settings a new entry in the file has.
    pub fn new(kind: WidgetKind, enabled: bool) -> Self {
        let mut widget = Self::of(&WidgetConfig::of_kind(kind));
        widget.enabled = enabled;
        widget
    }

    pub(crate) fn of(config: &WidgetConfig) -> Self {
        Self {
            kind: config.kind(),
            enabled: config.enabled(),
            format: config.format().map(str::to_owned),
            icon: config.icon().to_owned(),
            color: config.color().to_owned(),
        }
    }
}

/// The widgets of each side of the bar, in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BarWidgets {
    /// The widgets on the left.
    pub left: Vec<BarWidget>,
    /// The widgets on the right.
    pub right: Vec<BarWidget>,
}

impl BarWidgets {
    /// The widgets `config` has.
    pub fn of(config: &BarConfig) -> Self {
        let list = |widgets: &[WidgetConfig]| widgets.iter().map(BarWidget::of).collect();
        Self {
            left: list(&config.left),
            right: list(&config.right),
        }
    }

    /// The widgets on `zone`.
    pub fn zone(&self, zone: BarZone) -> &Vec<BarWidget> {
        match zone {
            BarZone::Left => &self.left,
            BarZone::Right => &self.right,
        }
    }

    /// The widgets on `zone`, to change.
    pub fn zone_mut(&mut self, zone: BarZone) -> &mut Vec<BarWidget> {
        match zone {
            BarZone::Left => &mut self.left,
            BarZone::Right => &mut self.right,
        }
    }

    /// Moves the widget at `index` of `from` to place `at` of `to`, kept
    /// within the list. Does nothing if there is no such widget.
    pub fn move_to(&mut self, from: BarZone, index: usize, to: BarZone, at: usize) {
        if index >= self.zone(from).len() {
            return;
        }
        let widget = self.zone_mut(from).remove(index);
        let target = self.zone_mut(to);
        target.insert(at.min(target.len()), widget);
    }
}

impl Default for BarWidgets {
    fn default() -> Self {
        Self::of(&BarConfig::default())
    }
}

#[cfg(test)]
#[path = "bar_widgets_tests.rs"]
mod tests;
