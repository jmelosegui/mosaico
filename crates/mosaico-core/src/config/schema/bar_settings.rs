//! The settings of `bar.toml`.

use super::{SettingDescriptor, SettingKind, SettingValue, config_settings::setting, default_at};
use crate::config::BarConfig;
use crate::config::limits;

const fn ranged((min, max): (i32, i32), unit: &'static str) -> SettingKind {
    SettingKind::Int {
        min: min as i64,
        max: max as i64,
        unit,
    }
}

const fn color(key: &'static str, label: &'static str, help: &'static str) -> SettingDescriptor {
    setting(key, label, help, SettingKind::Color)
}

/// Every setting of `bar.toml` that is a single value, in the order the
/// file has them. The monitors, the mouse bindings and the widgets are
/// lists, which the settings window edits on their own.
pub const BAR_SETTINGS: &[SettingDescriptor] = &[
    setting(
        "enabled",
        "Show the bar",
        "Whether the status bar is displayed.",
        SettingKind::Bool,
    ),
    setting(
        "height",
        "Height",
        "Increase for high-DPI displays.",
        ranged(limits::BAR_HEIGHT, "px"),
    ),
    setting(
        "font",
        "Family",
        "Installed fonts. The usual widget icons are drawn in lines; any other icon needs a font that has it, such as a Nerd Font.",
        SettingKind::Text,
    ),
    setting("font_size", "Size", "", ranged(limits::BAR_FONT_SIZE, "px")),
    setting("font_bold", "Bold", "", SettingKind::Bool),
    setting("font_italic", "Italic", "", SettingKind::Bool),
    setting("font_underline", "Underline", "", SettingKind::Bool),
    setting(
        "padding",
        "Edge padding",
        "Space between the bar's edges and its widgets.",
        ranged(limits::BAR_PADDING, "px"),
    ),
    setting(
        "pill_padding",
        "Pill padding",
        "Space inside each widget's pill.",
        ranged(limits::BAR_PILL_PADDING, "px"),
    ),
    setting(
        "pill_radius",
        "Pill radius",
        "0 gives square corners.",
        ranged(limits::BAR_PILL_RADIUS, "px"),
    ),
    setting(
        "pill_border_width",
        "Pill border",
        "0 gives no border.",
        ranged(limits::BAR_PILL_BORDER, "px"),
    ),
    setting(
        "item_gap",
        "Item gap",
        "",
        ranged(limits::BAR_ITEM_GAP, "px"),
    ),
    setting(
        "workspace_gap",
        "Workspace gap",
        "Space between the workspace numbers.",
        ranged(limits::BAR_WORKSPACE_GAP, "px"),
    ),
    setting(
        "separator",
        "Separator",
        "Text between the groups of widgets. Empty for none.",
        SettingKind::Text,
    ),
    setting(
        "background_opacity",
        "Background opacity",
        "0 is fully transparent and 100 is opaque.",
        ranged(limits::BAR_OPACITY, "%"),
    ),
    setting(
        "mouse.enabled",
        "Take clicks and the wheel",
        "Off lets the pointer through to the windows under the bar.",
        SettingKind::Bool,
    ),
    setting(
        "mouse.scroll",
        "Scrolling",
        "What the wheel does over the bar.",
        SettingKind::Choice(&[
            ("workspaces", "Step through workspaces"),
            ("none", "Do nothing"),
        ]),
    ),
    color("colors.background", "Background", ""),
    color("colors.foreground", "Text", ""),
    color(
        "colors.active_workspace",
        "Active workspace",
        "The pill of the workspace in use.",
    ),
    color("colors.active_workspace_text", "Active workspace text", ""),
    color("colors.inactive_workspace", "Other workspaces", ""),
    color("colors.separator", "Separator", ""),
    color("colors.accent", "Accent", "Alerts and update notices."),
    color("colors.widget_background", "Pill background", ""),
    color("colors.pill_border", "Pill border", "Empty for no border."),
];

/// What `bar.toml` has at `key` when the file leaves it out.
pub fn bar_default(key: &str) -> Option<SettingValue> {
    default_at(&BarConfig::default(), key)
}

/// The setting of `bar.toml` at `key`.
pub fn find_bar(key: &str) -> Option<&'static SettingDescriptor> {
    BAR_SETTINGS.iter().find(|s| s.key == key)
}
