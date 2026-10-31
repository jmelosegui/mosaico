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

/// A widget of the bar, without its own settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BarWidget {
    /// What the widget shows.
    pub kind: WidgetKind,
    /// Whether the bar shows it.
    pub enabled: bool,
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
        let list = |widgets: &[WidgetConfig]| {
            widgets
                .iter()
                .map(|w| BarWidget {
                    kind: w.kind(),
                    enabled: w.enabled(),
                })
                .collect()
        };
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
