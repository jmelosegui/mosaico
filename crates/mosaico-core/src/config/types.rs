/// Reusable type definitions for Mosaico configuration.
///
/// Contains layout, border, mouse, and corner-style types shared
/// across the configuration subsystem.
use serde::{Deserialize, Serialize};

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
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            gap: 8,
            ratio: 0.5,
            hiding: HidingBehaviour::default(),
        }
    }
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
    /// Hex color for the focused window border (e.g. "#00b4d8").
    pub focused: String,
    /// Hex color for the monocle mode border (e.g. "#2d6a4f").
    pub monocle: String,
}

/// Default border colors are empty — resolved from the theme in `validate()`.
impl Default for BorderConfig {
    fn default() -> Self {
        Self {
            width: 4,
            corner_style: CornerStyle::default(),
            focused: String::new(),
            monocle: String::new(),
        }
    }
}
