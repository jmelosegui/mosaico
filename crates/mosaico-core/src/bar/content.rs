//! What each bar shows, in drawing order, and what each item is to the
//! pointer.

use crate::WindowId;
use crate::config::bar::BarConfig;

use super::{BarSide, BarState, SystemReadings, WidgetClicks};

#[path = "content_items.rs"]
mod items;
#[path = "content_regions.rs"]
mod regions;

/// Everything one bar shows, in drawing order.
///
/// Built from the bar config, the monitor's [`BarState`] and the
/// [`SystemReadings`], so a platform only decides how it looks. Each item
/// also knows the widget it shows, and whether it takes clicks, which a
/// platform reads through [`region`](Self::region) and
/// [`label_region`](Self::label_region) as it draws.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BarContent {
    /// Drawn from the left edge, left to right.
    pub left: Vec<BarItem>,
    /// Drawn from the right edge, right to left.
    ///
    /// Only holds pills and separators. The workspaces and active window
    /// widgets are drawn as such only on the left, and as plain pills
    /// showing their icon on the right.
    pub right: Vec<BarItem>,
    /// The widget each item of `left` shows, by its index in the config's
    /// `left` list, in the same order, and `None` for a separator.
    left_widgets: Vec<Option<items::Shown>>,
    /// The same for `right`.
    right_widgets: Vec<Option<items::Shown>>,
    /// Whether a click on a workspace label goes to that workspace.
    labels_clickable: bool,
}

/// One thing drawn on a bar.
#[derive(Debug, Clone, PartialEq)]
pub enum BarItem {
    /// The workspace numbers in one container, with the one on screen
    /// highlighted. Only on the left side.
    Workspaces {
        /// The label of each workspace, in order.
        labels: Vec<String>,
        /// Index of the workspace on screen.
        active: usize,
    },
    /// The icon of the focused window's application. Only on the left
    /// side. Takes no space when no window is focused on this monitor, or
    /// its icon cannot be found.
    WindowIcon(Option<WindowId>),
    /// A widget's text in a pill.
    Pill {
        /// The widget's icon and text.
        label: String,
        /// Color of the text.
        text_color: String,
        /// Color of the pill's outline. Empty for none.
        border_color: String,
    },
    /// The configured separator between two widgets, with a space on each
    /// side.
    Separator(String),
}

impl BarContent {
    /// What a bar with `config` shows on a monitor in `state`.
    ///
    /// Each side lists its shown widgets in config order, with the
    /// separator between them when one is configured. The workspace labels
    /// take clicks while `[mouse] enabled` is on, and so does each widget
    /// a button runs an action on (see [`WidgetClicks`]).
    pub fn build(config: &BarConfig, state: &BarState, readings: &SystemReadings) -> Self {
        let clicks = WidgetClicks::new(&config.mouse);
        let (left, left_widgets) = items::side(config, BarSide::Left, state, readings, &clicks);
        let (right, right_widgets) = items::side(config, BarSide::Right, state, readings, &clicks);
        Self {
            left,
            right,
            left_widgets,
            right_widgets,
            labels_clickable: config.mouse.enabled,
        }
    }
}

#[cfg(test)]
#[path = "tests/content_tests.rs"]
mod tests;
