/// Window transitions: whether windows move to their new place, and how.
mod animations;
/// Status bar configuration.
pub mod bar;
/// Colors as the configuration writes them.
mod color;
/// How windows are hidden on a workspace switch.
mod hiding_behaviour;
/// Keybinding definitions and defaults.
pub mod keybinding;
/// What the mouse does: the pointer and focus, and dragging windows.
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
mod store;
/// Default configuration file templates.
pub mod template;
/// Theme resolution and color schemes.
pub mod theme;
/// Shared configuration types.
pub mod types;
/// The kinds of bar widget, by name.
mod widget_kind;
/// Workspace switching, stepping and per-workspace layouts.
mod workspaces;

use serde::{Deserialize, Serialize};

pub use animations::AnimationConfig;
pub use bar::{BarColors, BarConfig, BarMouseConfig, BarScroll, ClickBinding, WidgetConfig};
pub use color::Color;
pub use hiding_behaviour::HidingBehaviour;
pub use keybinding::{Keybinding, Modifier};
pub use mouse::MouseConfig;
pub use reload::{ConfigReload, ConfigReloader, ConfigWatch};
pub use rule_action::{RuleAction, decide};
pub use rules::{WindowRule, default_rules, matching_rule, validate_rules};
pub use store::ConfigStore;
pub use theme::{
    CatppuccinFlavor, RosePineFlavor, Theme, ThemeCheck, ThemeConfig, TokyoNightFlavor,
};
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
    /// Logging settings.
    pub logging: crate::log::LogConfig,
}

impl Config {
    /// Clamps layout, border and transition values to safe ranges and
    /// resolves theme colors for any unset border color fields.
    pub fn validate(&mut self) {
        self.resolve_borders();
        self.layout.gap = self.layout.gap.clamp(0, 200);
        self.layout.ratio = self.layout.ratio.clamp(0.1, 0.9);
        self.borders.width = self.borders.width.clamp(0, 32);
        let duration = self.animations.duration;
        self.animations.duration = duration.clamp(0, animations::MAX_DURATION_MS);
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
