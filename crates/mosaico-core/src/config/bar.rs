use serde::{Deserialize, Serialize};

use super::theme::Theme;

#[path = "bar_mouse.rs"]
mod mouse;

pub use mouse::{BarMouseConfig, BarScroll};

/// Top-level bar configuration.
///
/// Loaded from `~/.config/mosaico/bar.toml`. Missing fields fall back
/// to defaults thanks to `#[serde(default)]`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BarConfig {
    /// Whether the status bar is enabled.
    pub enabled: bool,
    /// Bar height in pixels.
    pub height: i32,
    /// Font family name (e.g. "JetBrainsMono Nerd Font").
    pub font: String,
    /// Font size in pixels for bar text.
    pub font_size: i32,
    /// Whether to render bar text in bold (weight 700 vs 400).
    pub font_bold: bool,
    /// Whether to render bar text in italic.
    pub font_italic: bool,
    /// Whether to render bar text with underline.
    pub font_underline: bool,
    /// Horizontal padding inside the bar (outer edges) in pixels.
    pub padding: i32,
    /// Horizontal padding inside each pill in pixels.
    pub pill_padding: i32,
    /// Corner radius for pill backgrounds in pixels. 0 = square.
    pub pill_radius: i32,
    /// Border width for pill backgrounds in pixels. 0 = no border.
    pub pill_border_width: i32,
    /// Gap between pills in pixels.
    pub item_gap: i32,
    /// Gap between individual workspace number pills in pixels.
    pub workspace_gap: i32,
    /// Separator string between widget groups (empty = no separator).
    pub separator: String,
    /// Background opacity as a percentage (0 = fully transparent, 100 = opaque).
    pub background_opacity: i32,
    /// Which monitors to show the bar on (0-indexed). Empty = all monitors.
    pub monitors: Vec<usize>,
    /// What the mouse does on the bar.
    pub mouse: BarMouseConfig,
    /// Bar color scheme. Empty fields are filled from the theme.
    pub colors: BarColors,
    /// Widgets displayed on the left side (rendered left-to-right).
    pub left: Vec<WidgetConfig>,
    /// Widgets displayed on the right side (rendered right-to-left).
    pub right: Vec<WidgetConfig>,
}

/// Color scheme for the status bar.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BarColors {
    /// Background color (hex, e.g. "#1e1e2e").
    pub background: String,
    /// Default text color (hex, e.g. "#cdd6f4").
    pub foreground: String,
    /// Color for the active workspace indicator.
    pub active_workspace: String,
    /// Text color inside the active workspace pill.
    pub active_workspace_text: String,
    /// Color for inactive workspace indicators.
    pub inactive_workspace: String,
    /// Separator color.
    pub separator: String,
    /// Accent color for alerts and update notifications.
    pub accent: String,
    /// Background color for widget pills.
    pub widget_background: String,
    /// Border color for widget pills (empty string = no border).
    #[serde(default)]
    pub pill_border: String,
}

/// Configuration for a single bar widget.
///
/// Each widget has a `type` field and optional settings. Set
/// `enabled = false` to hide a widget without removing its entry.
/// Set `color` to override text and border color (hex or named).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum WidgetConfig {
    /// Numbered workspace indicators with active highlight.
    Workspaces {
        /// Whether this widget is shown.
        #[serde(default = "default_true")]
        enabled: bool,
        /// Icon text prepended to the widget.
        #[serde(default)]
        icon: String,
        /// Text color (hex or named).
        #[serde(default)]
        color: String,
    },
    /// Current layout name (BSP) and monocle indicator.
    Layout {
        /// Whether this widget is shown.
        #[serde(default = "default_true")]
        enabled: bool,
        /// Icon text prepended to the widget.
        #[serde(default)]
        icon: String,
        /// Text color (hex or named).
        #[serde(default)]
        color: String,
    },
    /// Current time with configurable strftime format.
    Clock {
        /// Whether this widget is shown.
        #[serde(default = "default_true")]
        enabled: bool,
        /// Display format string.
        #[serde(default = "default_clock_format")]
        format: String,
        /// Icon text prepended to the widget.
        #[serde(default)]
        icon: String,
        /// Text color (hex or named).
        #[serde(default)]
        color: String,
    },
    /// Current date with configurable strftime format.
    Date {
        /// Whether this widget is shown.
        #[serde(default = "default_true")]
        enabled: bool,
        /// Display format string.
        #[serde(default = "default_date_format")]
        format: String,
        /// Icon text prepended to the widget.
        #[serde(default)]
        icon: String,
        /// Text color (hex or named).
        #[serde(default)]
        color: String,
    },
    /// System RAM usage percentage.
    Ram {
        /// Whether this widget is shown.
        #[serde(default = "default_true")]
        enabled: bool,
        /// Icon text prepended to the widget.
        #[serde(default)]
        icon: String,
        /// Text color (hex or named).
        #[serde(default)]
        color: String,
    },
    /// System CPU usage percentage.
    Cpu {
        /// Whether this widget is shown.
        #[serde(default = "default_true")]
        enabled: bool,
        /// Icon text prepended to the widget.
        #[serde(default)]
        icon: String,
        /// Text color (hex or named).
        #[serde(default)]
        color: String,
    },
    /// Update availability notification.
    Update {
        /// Whether this widget is shown.
        #[serde(default = "default_true")]
        enabled: bool,
        /// Icon text prepended to the widget.
        #[serde(default)]
        icon: String,
        /// Text color (hex or named).
        #[serde(default = "default_update_color")]
        color: String,
    },
    /// Icon of the currently focused window.
    #[serde(rename = "active_window")]
    ActiveWindow {
        /// Whether this widget is shown.
        #[serde(default = "default_true")]
        enabled: bool,
        /// Icon text prepended to the widget.
        #[serde(default)]
        icon: String,
        /// Text color (hex or named).
        #[serde(default)]
        color: String,
    },
    /// Currently playing media (artist and track title).
    Media {
        /// Whether this widget is shown.
        #[serde(default = "default_true")]
        enabled: bool,
        /// Icon text prepended to the widget.
        #[serde(default = "default_media_icon")]
        icon: String,
        /// Text color (hex or named).
        #[serde(default)]
        color: String,
        /// Maximum display length in characters.
        #[serde(default = "default_media_max_length")]
        max_length: usize,
    },
    /// Hotkey-paused indicator — only visible while mosaico hotkeys are paused.
    Paused {
        /// Whether this widget is shown.
        #[serde(default = "default_true")]
        enabled: bool,
        /// Icon text prepended to the widget.
        #[serde(default)]
        icon: String,
        /// Text color (hex or named). Defaults to red.
        #[serde(default = "default_paused_color")]
        color: String,
    },
}

fn default_true() -> bool {
    true
}

fn default_update_color() -> String {
    "green".into()
}

fn default_paused_color() -> String {
    "red".into()
}

impl WidgetConfig {
    /// Returns the icon string for this widget.
    pub fn icon(&self) -> &str {
        match self {
            Self::Workspaces { icon, .. }
            | Self::Layout { icon, .. }
            | Self::Clock { icon, .. }
            | Self::Date { icon, .. }
            | Self::Ram { icon, .. }
            | Self::Cpu { icon, .. }
            | Self::Update { icon, .. }
            | Self::ActiveWindow { icon, .. }
            | Self::Media { icon, .. }
            | Self::Paused { icon, .. } => icon,
        }
    }

    /// Returns whether this widget is enabled.
    pub fn enabled(&self) -> bool {
        match self {
            Self::Workspaces { enabled, .. }
            | Self::Layout { enabled, .. }
            | Self::Clock { enabled, .. }
            | Self::Date { enabled, .. }
            | Self::Ram { enabled, .. }
            | Self::Cpu { enabled, .. }
            | Self::Update { enabled, .. }
            | Self::ActiveWindow { enabled, .. }
            | Self::Media { enabled, .. }
            | Self::Paused { enabled, .. } => *enabled,
        }
    }

    /// Returns the custom color override for this widget (empty = use defaults).
    pub fn color(&self) -> &str {
        match self {
            Self::Workspaces { color, .. }
            | Self::Layout { color, .. }
            | Self::Clock { color, .. }
            | Self::Date { color, .. }
            | Self::Ram { color, .. }
            | Self::Cpu { color, .. }
            | Self::Update { color, .. }
            | Self::ActiveWindow { color, .. }
            | Self::Media { color, .. }
            | Self::Paused { color, .. } => color,
        }
    }

    /// Resolves the widget's custom color (named color name → hex).
    pub fn resolve_color_field(&mut self, theme: Theme) {
        let color = match self {
            Self::Workspaces { color, .. }
            | Self::Layout { color, .. }
            | Self::Clock { color, .. }
            | Self::Date { color, .. }
            | Self::Ram { color, .. }
            | Self::Cpu { color, .. }
            | Self::Update { color, .. }
            | Self::ActiveWindow { color, .. }
            | Self::Media { color, .. }
            | Self::Paused { color, .. } => color,
        };
        if !color.is_empty()
            && let Some(hex) = theme.named_color(color)
        {
            *color = hex.to_string();
        }
    }
}

fn default_media_icon() -> String {
    "\u{F001}".into()
}

fn default_media_max_length() -> usize {
    40
}

fn default_clock_format() -> String {
    "%H:%M:%S".into()
}

fn default_date_format() -> String {
    "%A %d %B %Y".into()
}

fn default_left_widgets() -> Vec<WidgetConfig> {
    vec![
        WidgetConfig::Workspaces {
            enabled: true,
            icon: String::new(),
            color: String::new(),
        },
        WidgetConfig::ActiveWindow {
            enabled: true,
            icon: String::new(),
            color: String::new(),
        },
        WidgetConfig::Layout {
            enabled: true,
            icon: "\u{F009}".into(),
            color: String::new(),
        },
    ]
}

fn default_right_widgets() -> Vec<WidgetConfig> {
    vec![
        WidgetConfig::Paused {
            enabled: true,
            icon: String::new(),
            color: default_paused_color(),
        },
        WidgetConfig::Clock {
            enabled: true,
            format: default_clock_format(),
            icon: "\u{F017}".into(),
            color: String::new(),
        },
        WidgetConfig::Date {
            enabled: true,
            format: default_date_format(),
            icon: "\u{F073}".into(),
            color: String::new(),
        },
        WidgetConfig::Ram {
            enabled: true,
            icon: "\u{F2DB}".into(),
            color: String::new(),
        },
        WidgetConfig::Cpu {
            enabled: true,
            icon: "\u{F085}".into(),
            color: String::new(),
        },
        WidgetConfig::Update {
            enabled: true,
            icon: "\u{F019}".into(),
            color: default_update_color(),
        },
    ]
}

impl Default for BarConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            height: 64,
            font: "CaskaydiaCove Nerd Font".into(),
            font_size: 24,
            font_bold: true,
            font_italic: false,
            font_underline: false,
            padding: 8,
            pill_padding: 12,
            pill_radius: 4,
            pill_border_width: 2,
            item_gap: 10,
            workspace_gap: 4,
            separator: String::new(),
            background_opacity: 0,
            monitors: Vec::new(),
            mouse: BarMouseConfig::default(),
            colors: BarColors::default(),
            left: default_left_widgets(),
            right: default_right_widgets(),
        }
    }
}

/// Default for BarColors uses empty strings so `resolve_theme()`
/// can detect which fields the user explicitly set.
impl Default for BarColors {
    fn default() -> Self {
        Self {
            background: String::new(),
            foreground: String::new(),
            active_workspace: String::new(),
            active_workspace_text: String::new(),
            inactive_workspace: String::new(),
            separator: String::new(),
            accent: String::new(),
            widget_background: String::new(),
            pill_border: String::new(),
        }
    }
}

impl BarConfig {
    /// Clamps bar values to safe ranges.
    pub fn validate(&mut self) {
        self.height = self.height.clamp(16, 96);
        self.font_size = self.font_size.clamp(8, 48);
        self.padding = self.padding.clamp(0, 64);
        self.pill_padding = self.pill_padding.clamp(0, 32);
        self.pill_radius = self.pill_radius.clamp(0, 32);
        self.pill_border_width = self.pill_border_width.clamp(0, 8);
        self.item_gap = self.item_gap.clamp(0, 32);
        self.workspace_gap = self.workspace_gap.clamp(0, 16);
        self.background_opacity = self.background_opacity.clamp(0, 100);
    }

    /// Fills empty color fields from the given theme, and resolves
    /// named colors (e.g. "blue") to their theme hex values.
    ///
    /// Any color the user explicitly set in `[colors]` is preserved.
    /// The theme is the global theme from `config.toml`.
    pub fn resolve_colors(&mut self, theme: Theme) {
        let palette = theme.bar_colors();
        let resolve = |field: &mut String, fallback: &str| {
            *field = theme.resolve_color(field, fallback).to_string();
        };
        resolve(&mut self.colors.background, &palette.background);
        resolve(&mut self.colors.foreground, &palette.foreground);
        resolve(&mut self.colors.active_workspace, &palette.active_workspace);
        resolve(
            &mut self.colors.active_workspace_text,
            &palette.active_workspace_text,
        );
        resolve(
            &mut self.colors.inactive_workspace,
            &palette.inactive_workspace,
        );
        resolve(&mut self.colors.separator, &palette.separator);
        resolve(&mut self.colors.accent, &palette.accent);
        resolve(
            &mut self.colors.widget_background,
            &palette.widget_background,
        );
        resolve(&mut self.colors.pill_border, &palette.pill_border);

        for w in &mut self.left {
            w.resolve_color_field(theme);
        }
        for w in &mut self.right {
            w.resolve_color_field(theme);
        }
    }

    /// Returns true if the bar should be displayed on the given monitor index.
    ///
    /// An empty `monitors` list means all monitors.
    pub fn should_show_on(&self, monitor_index: usize) -> bool {
        self.enabled && (self.monitors.is_empty() || self.monitors.contains(&monitor_index))
    }

    /// Returns true if an enabled CPU widget appears in the config.
    pub fn has_cpu_widget(&self) -> bool {
        let is_active_cpu = |w: &WidgetConfig| matches!(w, WidgetConfig::Cpu { enabled: true, .. });
        self.left.iter().any(is_active_cpu) || self.right.iter().any(is_active_cpu)
    }
}

#[cfg(test)]
#[path = "bar_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "bar_color_tests.rs"]
mod color_tests;

#[cfg(test)]
#[path = "bar_toml_tests.rs"]
mod toml_tests;
