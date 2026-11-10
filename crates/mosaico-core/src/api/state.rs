//! What a bar is shown of mosaico, grouped by topic.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::config::palette;
use crate::{StableMonitorId, WindowId};

/// Everything a bar is shown of mosaico, by topic.
///
/// `monitors` is always in `index` order.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
#[serde(default)]
pub struct State {
    /// The daemon itself.
    pub daemon: Daemon,
    /// The colors bars are drawn in.
    pub theme: Theme,
    /// Every connected monitor, from the left.
    pub monitors: Vec<Monitor>,
    /// Where the user is working.
    pub focus: Focus,
}

/// The daemon and what it was told.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
#[serde(default)]
pub struct Daemon {
    /// mosaico's version, such as `"0.14.0"`.
    pub version: String,
    /// The API version the state is shaped by.
    pub api: u32,
    /// The instance's name, or `None` for the default instance.
    pub instance: Option<String>,
    /// The bar the user picked, as `[bar] provider` names it.
    pub provider: String,
    /// Whether the hotkeys are paused.
    pub paused: bool,
    /// Notice of a newer version, such as `"v0.15.0 available"`, or
    /// `None` when there is none.
    pub update: Option<String>,
    /// How a workspace switch applies across monitors: `"per-monitor"` or
    /// `"global"`.
    pub workspace_mode: String,
    /// Whether stepping through the workspaces goes round from the last
    /// to the first.
    pub wrap: bool,
}

/// The theme bars are drawn in.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
#[serde(default)]
pub struct Theme {
    /// The theme's family, such as `"catppuccin"`.
    pub name: String,
    /// Its flavor, such as `"mocha"`.
    pub flavor: String,
    /// Its colors by name, such as `"blue": "#89b4fa"`.
    pub colors: BTreeMap<String, String>,
}

/// One connected monitor.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
#[serde(default)]
pub struct Monitor {
    /// The id the monitor keeps through display changes and reconnects.
    #[cfg_attr(test, schemars(with = "String"))]
    pub id: StableMonitorId,
    /// Position from the left, from 0, as `bar.toml`'s `monitors` counts.
    pub index: u32,
    /// What the system calls the display, such as `"DELL U2720Q"`.
    pub name: String,
    /// The name other programs know the display by.
    pub device: String,
    /// Scale factor, 1.0 at 100%.
    pub scale: f64,
    /// The whole monitor, in the desktop's pixels.
    pub bounds: Rect,
    /// What the system leaves for windows.
    pub work_area: Rect,
    /// Where mosaico tiles: the work area, kept clear of the bar.
    pub tiling_area: Rect,
    /// Whether a maximized or fullscreen window covers the monitor.
    pub maximized: bool,
    /// Number of the workspace on screen, from 1.
    pub active_workspace: u32,
    /// The monitor's workspaces, in number order.
    pub workspaces: Vec<Workspace>,
}

/// One workspace of a monitor.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
#[serde(default)]
pub struct Workspace {
    /// Number, from 1, as actions number workspaces.
    pub number: u32,
    /// What to call it. The number for now.
    pub name: String,
    /// Its layout, such as `"bsp"`, `"vertical-stack"` or
    /// `"three-column"`. A bar shows a layout it does not know as is.
    pub layout: String,
    /// Whether it shows one window over the whole area.
    pub monocle: bool,
    /// Its windows, in layout order.
    #[cfg_attr(test, schemars(with = "Vec<u64>"))]
    pub windows: Vec<WindowId>,
    /// The one of them focused last, if any.
    #[cfg_attr(test, schemars(with = "Option<u64>"))]
    pub focused: Option<WindowId>,
}

/// Where the user is working.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
#[serde(default)]
pub struct Focus {
    /// The monitor the user is working on.
    #[cfg_attr(test, schemars(with = "Option<String>"))]
    pub monitor: Option<StableMonitorId>,
    /// The number of the workspace on screen there.
    pub workspace: Option<u32>,
    /// The focused window, if any.
    pub window: Option<FocusedWindow>,
}

/// The focused window.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub struct FocusedWindow {
    /// Its id, for the session.
    #[cfg_attr(test, schemars(with = "u64"))]
    pub id: WindowId,
    /// Its title, cut at [`MAX_TITLE_CHARS`](super::MAX_TITLE_CHARS).
    /// Show it as text, never as markup.
    #[serde(default)]
    pub title: String,
    /// The file name of the program that owns it, or empty.
    #[serde(default)]
    pub program: String,
}

/// A rectangle in the desktop's pixels.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
#[serde(default)]
pub struct Rect {
    /// Left edge.
    pub x: i32,
    /// Top edge.
    pub y: i32,
    /// Width.
    pub width: i32,
    /// Height.
    pub height: i32,
}

impl From<crate::Rect> for Rect {
    fn from(rect: crate::Rect) -> Self {
        Self {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        }
    }
}

impl State {
    /// The monitor with id `id`, if connected.
    pub fn monitor(&self, id: &StableMonitorId) -> Option<&Monitor> {
        self.monitors.iter().find(|m| m.id == *id)
    }
}

impl Theme {
    /// The theme `theme`, with every color it names.
    pub fn of(theme: crate::config::Theme) -> Self {
        let colors = (palette::color_names(theme).into_iter())
            .filter_map(|name| {
                let hex = palette::named_color(theme, name)?;
                Some((name.to_string(), hex.to_string()))
            })
            .collect();
        Self {
            name: theme.name().to_string(),
            flavor: theme.flavor_name().to_string(),
            colors,
        }
    }
}
