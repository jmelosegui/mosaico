//! The items of one side of a bar.

use crate::config::bar::{BarConfig, WidgetConfig};

use super::super::widget_text::{is_shown, label};
use super::super::{BarSide, BarState, SystemReadings, WidgetClicks};
use super::BarItem;

/// The widget an item of a bar shows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Shown {
    /// The widget's index in the config's list of its side.
    pub(super) index: usize,
    /// Whether a click on it runs an action.
    pub(super) clickable: bool,
}

/// The items of one side of the bar, and the widget each one shows,
/// `None` for a separator. A widget is clickable when `clicks` has a
/// button run an action on it.
pub(super) fn side(
    config: &BarConfig,
    side: BarSide,
    state: &BarState,
    readings: &SystemReadings,
    clicks: &WidgetClicks,
) -> (Vec<BarItem>, Vec<Option<Shown>>) {
    let widgets = match side {
        BarSide::Left => &config.left,
        BarSide::Right => &config.right,
    };
    let mut items = Vec::new();
    let mut shown = Vec::new();
    for (index, widget) in widgets.iter().enumerate() {
        if !is_shown(widget, state, readings) {
            continue;
        }
        if !items.is_empty() && !config.separator.is_empty() {
            items.push(BarItem::Separator(format!(" {} ", config.separator)));
            shown.push(None);
        }
        items.push(item(config, widget, side, state, readings));
        let clickable = clicks.clickable(widget.kind(), side);
        shown.push(Some(Shown { index, clickable }));
    }
    (items, shown)
}

/// How `widget` is drawn.
///
/// Only the left side has the workspace container and the window icon.
/// On the right those widgets have always been plain pills showing their
/// icon.
fn item(
    config: &BarConfig,
    widget: &WidgetConfig,
    side: BarSide,
    state: &BarState,
    readings: &SystemReadings,
) -> BarItem {
    match widget {
        WidgetConfig::Workspaces { .. } if side == BarSide::Left => BarItem::Workspaces {
            labels: (1..=state.workspace_count).map(|n| n.to_string()).collect(),
            active: state.active_workspace,
        },
        WidgetConfig::ActiveWindow { .. } if side == BarSide::Left => {
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
        icon: None,
        label: label(widget, state, readings),
        text_color,
        border_color,
    }
}
