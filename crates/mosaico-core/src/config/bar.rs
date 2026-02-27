use serde::{Deserialize, Serialize};

use super::theme::Theme;

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
        #[serde(default = "default_true")]
        enabled: bool,
        #[serde(default)]
        icon: String,
        #[serde(default)]
        color: String,
    },
    /// Current layout name (BSP) and monocle indicator.
    Layout {
        #[serde(default = "default_true")]
        enabled: bool,
        #[serde(default)]
        icon: String,
        #[serde(default)]
        color: String,
    },
    /// Current time with configurable strftime format.
    Clock {
        #[serde(default = "default_true")]
        enabled: bool,
        #[serde(default = "default_clock_format")]
        format: String,
        #[serde(default)]
        icon: String,
        #[serde(default)]
        color: String,
    },
    /// Current date with configurable strftime format.
    Date {
        #[serde(default = "default_true")]
        enabled: bool,
        #[serde(default = "default_date_format")]
        format: String,
        #[serde(default)]
        icon: String,
        #[serde(default)]
        color: String,
    },
    /// System RAM usage percentage.
    Ram {
        #[serde(default = "default_true")]
        enabled: bool,
        #[serde(default)]
        icon: String,
        #[serde(default)]
        color: String,
    },
    /// System CPU usage percentage.
    Cpu {
        #[serde(default = "default_true")]
        enabled: bool,
        #[serde(default)]
        icon: String,
        #[serde(default)]
        color: String,
    },
    /// Update availability notification.
    Update {
        #[serde(default = "default_true")]
        enabled: bool,
        #[serde(default)]
        icon: String,
        #[serde(default = "default_update_color")]
        color: String,
    },
    /// Icon of the currently focused window.
    #[serde(rename = "active_window")]
    ActiveWindow {
        #[serde(default = "default_true")]
        enabled: bool,
        #[serde(default)]
        icon: String,
        #[serde(default)]
        color: String,
    },
}

fn default_true() -> bool {
    true
}

fn default_update_color() -> String {
    "green".into()
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
            | Self::ActiveWindow { icon, .. } => icon,
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
            | Self::ActiveWindow { enabled, .. } => *enabled,
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
            | Self::ActiveWindow { color, .. } => color,
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
            | Self::ActiveWindow { color, .. } => color,
        };
        if !color.is_empty()
            && let Some(hex) = theme.named_color(color)
        {
            *color = hex.to_string();
        }
    }
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
mod tests {
    use super::*;

    #[test]
    fn default_bar_config_has_expected_values() {
        let config = BarConfig::default();

        assert!(config.enabled);
        assert_eq!(config.height, 64);
        assert_eq!(config.font, "CaskaydiaCove Nerd Font");
        assert_eq!(config.font_size, 24);
        assert_eq!(config.pill_radius, 4);
        assert_eq!(config.pill_border_width, 2);
        assert!(config.font_bold);
        assert!(!config.font_italic);
        assert!(!config.font_underline);
        assert_eq!(config.background_opacity, 0);
        assert!(config.monitors.is_empty());
        assert_eq!(config.left.len(), 3);
        assert_eq!(config.right.len(), 5);
    }

    #[test]
    fn should_show_on_empty_monitors_means_all() {
        let config = BarConfig::default();
        assert!(config.should_show_on(0));
        assert!(config.should_show_on(1));
        assert!(config.should_show_on(99));
    }

    #[test]
    fn should_show_on_filters_by_index() {
        let config = BarConfig {
            monitors: vec![0, 2],
            ..Default::default()
        };
        assert!(config.should_show_on(0));
        assert!(!config.should_show_on(1));
        assert!(config.should_show_on(2));
    }

    #[test]
    fn should_show_on_respects_enabled() {
        let config = BarConfig {
            enabled: false,
            ..Default::default()
        };
        assert!(!config.should_show_on(0));
    }

    #[test]
    fn monitors_field_round_trips_through_toml() {
        let config = BarConfig {
            monitors: vec![0, 2],
            ..Default::default()
        };
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: BarConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.monitors, vec![0, 2]);
    }

    #[test]
    fn monitors_field_from_toml() {
        let config: BarConfig = toml::from_str("monitors = [1]\n").unwrap();
        assert_eq!(config.monitors, vec![1]);
        assert!(!config.should_show_on(0));
        assert!(config.should_show_on(1));
    }

    #[test]
    fn resolve_colors_fills_from_theme() {
        let mut config = BarConfig::default();
        config.resolve_colors(Theme::Mocha);

        assert_eq!(config.colors.background, "#1e1e2e");
        assert_eq!(config.colors.widget_background, "#313244");
        assert_eq!(config.colors.active_workspace_text, "#cdd6f4");
    }

    #[test]
    fn bar_config_round_trips_through_toml() {
        let mut config = BarConfig::default();
        config.resolve_colors(Theme::Mocha);
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: BarConfig = toml::from_str(&toml_str).unwrap();

        assert_eq!(parsed.height, config.height);
        assert_eq!(parsed.font, config.font);
        assert_eq!(parsed.pill_radius, config.pill_radius);
        assert_eq!(parsed.left.len(), config.left.len());
        assert_eq!(parsed.right.len(), config.right.len());
    }

    #[test]
    fn empty_toml_uses_all_defaults() {
        let config: BarConfig = toml::from_str("").unwrap();

        assert!(config.enabled);
        assert_eq!(config.height, 64);
        assert_eq!(config.font, "CaskaydiaCove Nerd Font");
        assert_eq!(config.left.len(), 3);
    }

    #[test]
    fn partial_toml_preserves_unset_defaults() {
        let config: BarConfig = toml::from_str("height = 40\nfont = \"Hack\"\n").unwrap();

        assert_eq!(config.height, 40);
        assert_eq!(config.font, "Hack");
        assert_eq!(config.font_size, 24);
    }

    #[test]
    fn widget_with_icon_deserializes() {
        let toml_str = "[[left]]\ntype = \"layout\"\nicon = \"X\"\n";
        let config: BarConfig = toml::from_str(toml_str).unwrap();

        assert_eq!(config.left.len(), 1);
        assert_eq!(config.left[0].icon(), "X");
    }

    #[test]
    fn widget_without_icon_defaults_to_empty() {
        let toml_str = "[[left]]\ntype = \"layout\"\n";
        let config: BarConfig = toml::from_str(toml_str).unwrap();

        assert_eq!(config.left[0].icon(), "");
    }

    #[test]
    fn clock_with_icon_and_format() {
        let toml_str = r#"
            [[right]]
            type = "clock"
            format = "%H:%M"
            icon = "C"
        "#;
        let config: BarConfig = toml::from_str(toml_str).unwrap();

        match &config.right[0] {
            WidgetConfig::Clock { format, icon, .. } => {
                assert_eq!(format, "%H:%M");
                assert_eq!(icon, "C");
            }
            other => panic!("expected Clock, got {other:?}"),
        }
    }

    #[test]
    fn validate_clamps_extreme_values() {
        let mut config = BarConfig {
            height: 200,
            font_size: 0,
            padding: -5,
            pill_radius: 100,
            background_opacity: 150,
            ..Default::default()
        };

        config.validate();

        assert_eq!(config.height, 96);
        assert_eq!(config.font_size, 8);
        assert_eq!(config.padding, 0);
        assert_eq!(config.pill_radius, 32);
        assert_eq!(config.background_opacity, 100);
    }

    #[test]
    fn validate_preserves_valid_values() {
        let mut config = BarConfig::default();
        config.validate();
        assert_eq!(config.height, 64);
        assert_eq!(config.pill_radius, 4);
    }

    #[test]
    fn resolve_colors_latte_fills_light_colors() {
        let mut config = BarConfig::default();
        config.resolve_colors(Theme::Latte);
        assert_eq!(config.colors.background, "#eff1f5");
        assert_eq!(config.colors.foreground, "#1e66f5");
    }

    #[test]
    fn explicit_color_overrides_theme() {
        let toml_str = "[colors]\nbackground = \"#000000\"\n";
        let mut config: BarConfig = toml::from_str(toml_str).unwrap();
        config.resolve_colors(Theme::Latte);

        // Explicit override kept
        assert_eq!(config.colors.background, "#000000");
        // Unset fields resolved from latte
        assert_eq!(config.colors.foreground, "#1e66f5");
    }

    #[test]
    fn named_color_in_bar_resolves_to_hex() {
        let toml_str = "[colors]\naccent = \"mauve\"\n";
        let mut config: BarConfig = toml::from_str(toml_str).unwrap();
        config.resolve_colors(Theme::Mocha);

        assert_eq!(config.colors.accent, "#cba6f7");
        // Unset fields still resolved from theme
        assert_eq!(config.colors.background, "#1e1e2e");
    }

    #[test]
    fn widget_color_resolves_named_to_hex() {
        let mut config = BarConfig::default();
        config.resolve_colors(Theme::Mocha);
        // Update widget defaults to "green", resolved to Mocha green hex.
        let update = config
            .right
            .iter()
            .find(|w| matches!(w, WidgetConfig::Update { .. }))
            .unwrap();
        assert_eq!(update.color(), "#a6e3a1");
    }

    #[test]
    fn widget_color_empty_means_no_override() {
        let mut config = BarConfig::default();
        config.resolve_colors(Theme::Mocha);
        // Clock has no custom color — stays empty.
        let clock = config
            .right
            .iter()
            .find(|w| matches!(w, WidgetConfig::Clock { .. }))
            .unwrap();
        assert!(clock.color().is_empty());
    }

    #[test]
    fn widget_color_from_toml() {
        let toml_str = "[[left]]\ntype = \"layout\"\ncolor = \"red\"\n";
        let mut config: BarConfig = toml::from_str(toml_str).unwrap();
        config.resolve_colors(Theme::Mocha);
        assert_eq!(config.left[0].color(), "#f38ba8");
    }
}
