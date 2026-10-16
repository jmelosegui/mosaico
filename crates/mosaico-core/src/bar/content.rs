//! What each bar shows, in drawing order.

use crate::WindowId;
use crate::config::bar::{BarConfig, WidgetConfig};

use super::widget_text::{is_shown, label};
use super::{BarState, SystemReadings};

/// Everything one bar shows, in drawing order.
///
/// Built from the bar config, the monitor's [`BarState`] and the
/// [`SystemReadings`], so a platform only decides how it looks.
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

/// Which side of the bar a widget is on.
#[derive(Clone, Copy, PartialEq)]
enum Side {
    Left,
    Right,
}

impl BarContent {
    /// What a bar with `config` shows on a monitor in `state`.
    ///
    /// Each side lists its shown widgets in config order, with the
    /// separator between them when one is configured.
    pub fn build(config: &BarConfig, state: &BarState, readings: &SystemReadings) -> Self {
        Self {
            left: side(config, &config.left, Side::Left, state, readings),
            right: side(config, &config.right, Side::Right, state, readings),
        }
    }
}

/// The items of one side of the bar.
fn side(
    config: &BarConfig,
    widgets: &[WidgetConfig],
    side: Side,
    state: &BarState,
    readings: &SystemReadings,
) -> Vec<BarItem> {
    let mut items = Vec::new();
    for widget in widgets.iter().filter(|w| is_shown(w, state, readings)) {
        if !items.is_empty() && !config.separator.is_empty() {
            items.push(BarItem::Separator(format!(" {} ", config.separator)));
        }
        items.push(item(config, widget, side, state, readings));
    }
    items
}

/// How `widget` is drawn.
///
/// Only the left side has the workspace container and the window icon.
/// On the right those widgets have always been plain pills showing their
/// icon.
fn item(
    config: &BarConfig,
    widget: &WidgetConfig,
    side: Side,
    state: &BarState,
    readings: &SystemReadings,
) -> BarItem {
    match widget {
        WidgetConfig::Workspaces { .. } if side == Side::Left => BarItem::Workspaces {
            labels: (1..=state.workspace_count).map(|n| n.to_string()).collect(),
            active: state.active_workspace,
        },
        WidgetConfig::ActiveWindow { .. } if side == Side::Left => {
            BarItem::WindowIcon(state.focused_window)
        }
        _ => pill(config, widget, state, readings),
    }
}

/// A pill with the widget's label, in the widget's color when it has one
/// and in the bar's colors otherwise.
fn pill(
    config: &BarConfig,
    widget: &WidgetConfig,
    state: &BarState,
    readings: &SystemReadings,
) -> BarItem {
    let color = widget.color();
    let (text_color, border_color) = if color.is_empty() {
        (
            config.colors.foreground.clone(),
            config.colors.pill_border.clone(),
        )
    } else {
        (color.to_string(), color.to_string())
    };
    BarItem::Pill {
        label: label(widget, state, readings),
        text_color,
        border_color,
    }
}

#[cfg(test)]
#[path = "tests/content_tests.rs"]
mod tests;
