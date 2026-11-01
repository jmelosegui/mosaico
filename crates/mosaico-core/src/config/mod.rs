/// Window transitions: whether windows move to their new place, and how.
mod animations;
/// Status bar configuration.
pub mod bar;
/// Which widgets the bar shows, where, and in what order.
mod bar_widgets;
/// Colors as the configuration writes them.
mod color;
/// How windows are hidden on a workspace switch.
mod hiding_behaviour;
/// Keybinding definitions and defaults.
pub mod keybinding;
mod keybinding_conflicts;
/// What the mouse does: the pointer and focus, and dragging windows.
pub mod limits;
mod mouse;
/// Color palette definitions for themes.
mod palette;
/// Which config files the daemon reloads while it runs, and when.
mod reload;
/// Telling the user about their config while it loads.
mod report;
/// What a window rule does with the windows it matches.
mod rule_action;
/// Window management rules.
pub mod rules;
/// The config files of an instance: where they are, and loading,
/// merging and migrating them.
pub mod schema;
/// What the settings window remembers between runs.
mod settings_app;
mod store;
/// Default configuration file templates.
pub mod template;
/// Theme resolution and color schemes.
pub mod theme;
/// The icon in the notification area.
mod tray;
/// Shared configuration types.
pub mod types;
/// The kinds of bar widget, by name.
mod widget_kind;
/// Workspace switching, stepping and per-workspace layouts.
mod workspaces;

use serde::{Deserialize, Serialize};

pub use animations::AnimationConfig;
pub use bar::{BarColors, BarConfig, BarMouseConfig, BarScroll, ClickBinding, WidgetConfig};
pub use bar_widgets::{BarWidget, BarWidgets, BarZone};
pub use color::Color;
pub use hiding_behaviour::HidingBehaviour;
pub use keybinding::{Keybinding, Modifier};
pub use keybinding_conflicts::{Conflict, conflicts};
pub use mouse::MouseConfig;
pub use palette::UiColors;
pub use reload::{ConfigReload, ConfigReloader, ConfigWatch};
pub use rule_action::{RuleAction, decide};
pub use rules::{WindowRule, default_rules, matching_rule, validate_rules};
pub use settings_app::{SETTINGS_WINDOW_MIN, SETTINGS_WINDOW_SIZE, SettingsAppConfig};
pub use store::{ConfigStore, DefaultFile, KeybindingsDocument, RulesDocument, SettingsDocument};
pub use theme::{
    CatppuccinFlavor, RosePineFlavor, Theme, ThemeCheck, ThemeConfig, TokyoNightFlavor,
};
pub use tray::TrayConfig;
pub use types::*;
pub use widget_kind::WidgetKind;
pub use workspaces::{WorkspaceMode, WorkspacesConfig};

/// Top-level configuration for Mosaico.
///
/// Loaded from `~/.config/mosaico/config.toml`. Missing sections
/// fall back to defaults thanks to `#[serde(default)]`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Color theme (e.g. `[theme] name = "catppuccin" flavor = "mocha"`).
    pub theme: ThemeConfig,
    /// Layout algorithm parameters.
    pub layout: LayoutConfig,
    /// Workspace switching mode and per-workspace layout overrides.
    pub workspaces: WorkspacesConfig,
    /// Border appearance settings.
    pub borders: BorderConfig,
    /// Mouse integration settings.
    pub mouse: MouseConfig,
    /// Window transitions.
    pub animations: AnimationConfig,
    /// The icon in the notification area.
    pub tray: TrayConfig,
    /// Logging settings.
    pub logging: crate::log::LogConfig,
}

impl Config {
    /// Clamps layout, border and transition values to safe ranges and
    /// resolves theme colors for any unset border color fields.
    pub fn validate(&mut self) {
        self.resolve_borders();
        self.layout.gap = self.layout.gap.clamp(0, limits::GAP_MAX);
        let ratio = self.layout.ratio;
        self.layout.ratio = ratio.clamp(limits::RATIO_MIN, limits::RATIO_MAX);
        self.borders.width = self.borders.width.clamp(0, limits::BORDER_WIDTH_MAX);
        let duration = self.animations.duration;
        self.animations.duration = duration.clamp(0, limits::DURATION_MAX_MS);
    }

    /// Resolves border colors: empty -> theme default, named -> theme hex.
    ///
    /// `unfocused` accepts a sentinel value of `BorderColors::NONE`
    /// ("none") meaning "do not draw borders around unfocused windows";
    /// the sentinel is preserved verbatim so downstream code can detect
    /// it via `BorderColors::unfocused_enabled()`.
    fn resolve_borders(&mut self) {
        let theme = self.theme.resolve();
        self.borders.colors.focused = theme
            .resolve_color(&self.borders.colors.focused, theme.border_focused())
            .to_string();
        self.borders.colors.monocle = theme
            .resolve_color(&self.borders.colors.monocle, theme.border_monocle())
            .to_string();
        if self.borders.colors.unfocused != BorderColors::NONE {
            self.borders.colors.unfocused = theme
                .resolve_color(&self.borders.colors.unfocused, theme.border_unfocused())
                .to_string();
        }
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
