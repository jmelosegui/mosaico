/// Reusable type definitions for Mosaico configuration.
///
/// Contains layout, border, mouse, and corner-style types shared
/// across the configuration subsystem.
use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize};

use crate::action::MAX_WORKSPACES;
use crate::layout::LayoutKind;

/// Layout algorithm settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LayoutConfig {
    /// Gap in pixels between windows and screen edges.
    pub gap: i32,
    /// Ratio of space given to the first window in each split (0.0–1.0).
    pub ratio: f64,
    /// How windows are hidden during workspace switches.
    pub hiding: HidingBehaviour,
    /// Default layout for workspaces without an explicit override.
    pub default: LayoutKind,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            gap: 8,
            ratio: 0.5,
            hiding: HidingBehaviour::default(),
            default: LayoutKind::default(),
        }
    }
}

/// Workspace-level configuration.
///
/// Holds the workspace switching mode and per-workspace layout
/// overrides. See [`WorkspaceMode`] for the available modes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct WorkspacesConfig {
    /// How a workspace switch is applied across monitors.
    pub mode: WorkspaceMode,
    /// Per-workspace layout overrides (workspace number 1-8 to layout).
    ///
    /// TOML keys are always strings, so we deserialize via a helper that
    /// parses each key as a `u8` and silently drops keys outside the
    /// valid `1..=MAX_WORKSPACES` range (logging a warning for each).
    /// This lets users write `[workspaces.layouts]\n1 = "vertical-stack"`
    /// while the internal lookup remains `u8`-keyed.
    #[serde(deserialize_with = "deserialize_layouts_map")]
    pub layouts: HashMap<u8, LayoutKind>,
}

/// Deserializes a TOML table whose string keys represent workspace
/// numbers (`1`..=`MAX_WORKSPACES`). Invalid keys are dropped with a
/// warning so a typo on one workspace never fails the whole config.
fn deserialize_layouts_map<'de, D>(deserializer: D) -> Result<HashMap<u8, LayoutKind>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw: HashMap<String, LayoutKind> = HashMap::deserialize(deserializer)?;
    let mut out = HashMap::with_capacity(raw.len());
    for (key, value) in raw {
        match key.parse::<u8>() {
            Ok(n) if (1..=MAX_WORKSPACES).contains(&n) => {
                out.insert(n, value);
            }
            Ok(n) => {
                eprintln!(
                    "Warning: ignoring [workspaces.layouts] entry for workspace {n}: \
                     must be in 1..={MAX_WORKSPACES}"
                );
            }
            Err(_) => {
                eprintln!(
                    "Warning: ignoring [workspaces.layouts] entry with non-numeric key {key:?}"
                );
            }
        }
    }
    Ok(out)
}

/// How windows are hidden when switching away from their workspace.
#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HidingBehaviour {
    /// DWM Cloak: window becomes invisible but keeps its taskbar icon
    /// and does not fire `EVENT_OBJECT_HIDE`. Recommended default.
    #[default]
    Cloak,
    /// `ShowWindow(SW_HIDE)`: window is fully hidden and loses its
    /// taskbar icon. Fires `EVENT_OBJECT_HIDE`.
    Hide,
    /// `ShowWindow(SW_MINIMIZE)`: window is minimized. Keeps taskbar
    /// icon but shows minimized state. Fires `EVENT_SYSTEM_MINIMIZESTART`.
    Minimize,
}

/// Corner style for borders and tiled windows.
///
/// Controls both the border overlay shape (pixel-exact rounding) and
/// the DWM corner preference applied to managed windows on Windows 11.
#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CornerStyle {
    /// Sharp rectangular corners (DWM: `DONOTROUND`).
    Square,
    /// Subtle rounding (border: 8 px, DWM: `ROUNDSMALL` ~4 px).
    #[default]
    Small,
    /// Standard rounding (border: 16 px, DWM: `ROUND` ~8 px).
    Round,
}

impl CornerStyle {
    /// Pixel radius used for the border overlay rasterization.
    pub fn border_radius(self) -> i32 {
        match self {
            Self::Square => 0,
            Self::Small => 8,
            Self::Round => 16,
        }
    }
}

/// Mouse integration settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MouseConfig {
    /// Move the cursor to the center of the focused window on
    /// keyboard navigation.
    pub follows_focus: bool,
    /// Automatically focus the window under the cursor without clicking.
    pub focus_follows_mouse: bool,
}

impl Default for MouseConfig {
    fn default() -> Self {
        Self {
            follows_focus: true,
            focus_follows_mouse: false,
        }
    }
}

/// Border appearance configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BorderConfig {
    /// Border width in pixels.
    pub width: i32,
    /// Corner style for borders and tiled windows.
    pub corner_style: CornerStyle,
    /// Border colors for focused windows in different layout modes.
    pub colors: BorderColors,
}

/// Default border colors are empty — resolved from the theme in `validate()`.
impl Default for BorderConfig {
    fn default() -> Self {
        Self {
            width: 4,
            corner_style: CornerStyle::default(),
            colors: BorderColors::default(),
        }
    }
}

/// Border colors for the focused window across layout modes.
///
/// Both fields hold a hex string (e.g. `"#00b4d8"`) or a named theme
/// color. Empty strings resolve to the active theme's defaults during
/// validation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct BorderColors {
    /// Focused-window border color while a tiled layout is active.
    pub focused: String,
    /// Focused-window border color while monocle mode is active.
    pub monocle: String,
}

/// How a workspace switch is applied across monitors.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum WorkspaceMode {
    /// Switching workspaces only affects the focused monitor (default).
    #[default]
    PerMonitor,
    /// Switching workspaces affects every monitor in lockstep, mirroring
    /// how Windows virtual desktops work.
    Global,
}
