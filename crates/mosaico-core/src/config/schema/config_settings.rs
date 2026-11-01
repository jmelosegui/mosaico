//! The settings of `config.toml`.

use super::{SettingDescriptor, SettingKind, SettingValue};
use crate::config::Config;
use crate::config::limits::{
    BORDER_WIDTH_MAX, DURATION_MAX_MS, GAP_MAX, LOG_FILE_MB, RATIO_MAX, RATIO_MIN,
};

const LAYOUTS: &[(&str, &str)] = &[
    ("bsp", "BSP"),
    ("vertical-stack", "Vertical stack"),
    ("three-column", "Three column"),
];

/// The layouts a workspace can have of its own; the empty name is none,
/// so the default layout applies.
const WORKSPACE_LAYOUTS: &[(&str, &str)] = &[
    ("", "Default layout"),
    ("bsp", "BSP"),
    ("vertical-stack", "Vertical stack"),
    ("three-column", "Three column"),
];

/// The settings that are the entries of `[workspaces.layouts]`, which
/// the default config leaves empty.
const WORKSPACE_LAYOUT_PREFIX: &str = "workspaces.layouts.";

const LOG_LEVELS: &[(&str, &str)] = &[
    ("debug", "Debug"),
    ("info", "Info"),
    ("warn", "Warn"),
    ("error", "Error"),
];

const fn setting(
    key: &'static str,
    label: &'static str,
    help: &'static str,
    kind: SettingKind,
) -> SettingDescriptor {
    SettingDescriptor {
        key,
        label,
        help,
        kind,
    }
}

const fn workspace_layout(key: &'static str, label: &'static str) -> SettingDescriptor {
    setting(
        key,
        label,
        "The layout of this workspace. The default layout applies when none is picked.",
        SettingKind::Choice(WORKSPACE_LAYOUTS),
    )
}

const fn pixels(max: i32) -> SettingKind {
    SettingKind::Int {
        min: 0,
        max: max as i64,
        unit: "px",
    }
}

/// Every setting of `config.toml`, in the order the file has them.
///
/// The layout of a workspace is empty while `[workspaces.layouts]` has no
/// entry for it, which is its default, so picking none removes the entry.
pub const CONFIG_SETTINGS: &[SettingDescriptor] = &[
    setting(
        "theme.name",
        "Theme",
        "Colors for window borders, the bar and the settings window.",
        SettingKind::Text,
    ),
    setting("theme.flavor", "Flavor", "", SettingKind::Text),
    setting(
        "layout.gap",
        "Gap",
        "Space between windows and around the screen edges.",
        pixels(GAP_MAX),
    ),
    setting(
        "layout.ratio",
        "Split ratio",
        "The share of a split the first window gets.",
        SettingKind::Float {
            min: RATIO_MIN,
            max: RATIO_MAX,
            step: 0.05,
        },
    ),
    setting(
        "layout.hiding",
        "Hide windows by",
        "How windows on other workspaces are hidden. Cloak keeps their taskbar entries.",
        SettingKind::Choice(&[
            ("cloak", "Cloak"),
            ("hide", "Hide"),
            ("minimize", "Minimize"),
        ]),
    ),
    setting(
        "layout.default",
        "Default layout",
        "The layout of a workspace that has none of its own.",
        SettingKind::Choice(LAYOUTS),
    ),
    setting(
        "workspaces.mode",
        "Switching",
        "Whether a switch changes the focused monitor or every monitor.",
        SettingKind::Choice(&[
            ("per-monitor", "Focused monitor"),
            ("global", "All monitors"),
        ]),
    ),
    setting(
        "workspaces.wrap",
        "Wrap around",
        "Stepping past the last workspace goes back to the first.",
        SettingKind::Bool,
    ),
    workspace_layout("workspaces.layouts.1", "Workspace 1"),
    workspace_layout("workspaces.layouts.2", "Workspace 2"),
    workspace_layout("workspaces.layouts.3", "Workspace 3"),
    workspace_layout("workspaces.layouts.4", "Workspace 4"),
    workspace_layout("workspaces.layouts.5", "Workspace 5"),
    workspace_layout("workspaces.layouts.6", "Workspace 6"),
    workspace_layout("workspaces.layouts.7", "Workspace 7"),
    workspace_layout("workspaces.layouts.8", "Workspace 8"),
    setting(
        "borders.width",
        "Width",
        "How thick the border around windows is. 0 draws none.",
        pixels(BORDER_WIDTH_MAX),
    ),
    setting(
        "borders.corner_style",
        "Corners",
        "How round borders and window corners are.",
        SettingKind::Choice(&[("square", "Square"), ("small", "Small"), ("round", "Round")]),
    ),
    setting(
        "borders.colors.focused",
        "Focused",
        "The border of the focused window.",
        SettingKind::Color,
    ),
    setting(
        "borders.colors.monocle",
        "Monocle",
        "The border of the focused window in monocle.",
        SettingKind::Color,
    ),
    setting(
        "borders.colors.unfocused",
        "Unfocused",
        "The border of the other windows. None draws no border.",
        SettingKind::Color,
    ),
    setting(
        "mouse.follows_focus",
        "Pointer follows focus",
        "Moving focus with the keyboard moves the pointer to the window.",
        SettingKind::Bool,
    ),
    setting(
        "mouse.focus_follows_mouse",
        "Focus follows pointer",
        "Pointing at a window focuses it.",
        SettingKind::Bool,
    ),
    setting(
        "mouse.drag_to_swap",
        "Drag to swap",
        "Dropping a dragged window on another swaps them.",
        SettingKind::Bool,
    ),
    setting(
        "mouse.resize_splits",
        "Resize splits",
        "Resizing a window with the mouse moves the split next to it.",
        SettingKind::Bool,
    ),
    setting(
        "animations.enabled",
        "Animate windows",
        "Windows move to their new place instead of jumping there.",
        SettingKind::Bool,
    ),
    setting(
        "animations.duration",
        "Duration",
        "How long a transition takes.",
        SettingKind::Int {
            min: 0,
            max: DURATION_MAX_MS as i64,
            unit: "ms",
        },
    ),
    setting(
        "animations.easing",
        "Easing",
        "The curve windows follow: a preset or cubic-bezier(x1, y1, x2, y2).",
        SettingKind::Text,
    ),
    setting(
        "animations.follow_system",
        "Follow the system",
        "Windows snap while the system turns animations off.",
        SettingKind::Bool,
    ),
    setting(
        "tray.enabled",
        "Show the tray icon",
        "Shows mosaico in the notification area.",
        SettingKind::Bool,
    ),
    setting(
        "logging.enabled",
        "Write a log file",
        "Logs go to the mosaico logs folder.",
        SettingKind::Bool,
    ),
    setting(
        "logging.level",
        "Level",
        "The least severe messages that are written.",
        SettingKind::Choice(LOG_LEVELS),
    ),
    setting(
        "logging.max_file_mb",
        "Maximum file size",
        "The log is rotated when it reaches this size.",
        SettingKind::Int {
            min: LOG_FILE_MB.0 as i64,
            max: LOG_FILE_MB.1 as i64,
            unit: "MB",
        },
    ),
];

/// What `config.toml` has at `key` when the file leaves it out.
pub fn config_default(key: &str) -> Option<SettingValue> {
    if key.starts_with(WORKSPACE_LAYOUT_PREFIX) {
        return super::find(key).map(|_| SettingValue::Text(String::new()));
    }
    let mut value = toml::Value::try_from(Config::default()).ok()?;
    for part in key.split('.') {
        value = value.as_table_mut()?.remove(part)?;
    }
    SettingValue::from_toml(&value)
}
