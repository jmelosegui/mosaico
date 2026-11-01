/// Reusable type definitions for Mosaico configuration.
///
/// Contains layout, border and corner-style types shared across the
/// configuration subsystem.
use serde::{Deserialize, Serialize};

use crate::config::Color;
use crate::layout::LayoutKind;

use super::HidingBehaviour;

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
            gap: 20,
            ratio: 0.5,
            hiding: HidingBehaviour::default(),
            default: LayoutKind::default(),
        }
    }
}

/// Corner style for borders and tiled windows.
///
/// Sets how round the borders drawn around windows are, and how round
/// the corners of managed windows are on platforms that let windows
/// choose.
#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CornerStyle {
    /// Sharp, square corners.
    Square,
    /// Slightly rounded corners: an 8 px border radius, and the
    /// platform's small rounding on windows.
    #[default]
    Small,
    /// More rounded corners: a 16 px border radius, and the platform's
    /// standard rounding on windows.
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

/// Border colors across focus and layout states.
///
/// Each field holds a hex string (e.g. `"#00b4d8"`) or a named theme
/// color. Empty strings resolve to the active theme's defaults during
/// validation. The `unfocused` field also accepts the literal value
/// `"none"` to disable rendering borders around unfocused windows
/// (only the focused window will then have a border). A value that is
/// still not a hex color after that, such as a name the theme does not
/// know, is drawn in [`BorderColors::FALLBACK`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct BorderColors {
    /// Focused-window border color while a tiled layout is active.
    pub focused: String,
    /// Focused-window border color while monocle mode is active.
    pub monocle: String,
    /// Border color drawn around unfocused tiled windows. Empty string
    /// uses the theme default (a muted gray); `"none"` disables
    /// unfocused borders entirely.
    pub unfocused: String,
}

impl BorderColors {
    /// Sentinel value for `unfocused` meaning "no border around
    /// unfocused windows".
    pub const NONE: &'static str = "none";

    /// The color a border is drawn in when its configured value is not
    /// a hex color. Matches the historical default focused color.
    pub const FALLBACK: Color = Color {
        r: 0x00,
        g: 0xB4,
        b: 0xD8,
    };

    /// Returns true when unfocused borders should be rendered.
    pub fn unfocused_enabled(&self) -> bool {
        self.unfocused != Self::NONE
    }
}
