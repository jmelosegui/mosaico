pub mod bar;
pub mod keybinding;
mod loader;
mod palette;
pub mod rules;
pub mod template;
pub mod theme;
pub mod types;

use serde::{Deserialize, Serialize};

pub use bar::{BarColors, BarConfig, WidgetConfig};
pub use keybinding::{Keybinding, Modifier};
pub use loader::{
    bar_path, config_dir, config_path, keybindings_path, load, load_bar, load_keybindings,
    load_merged_rules, load_rules, load_user_rules, rules_path, try_load, try_load_bar,
    try_load_keybindings, try_load_rules, try_load_user_rules, user_rules_path,
};
pub use rules::{WindowRule, default_rules, should_manage, validate_rules};
pub use theme::{Theme, ThemeConfig};
pub use types::*;

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
    /// Border appearance settings.
    pub borders: BorderConfig,
    /// Mouse integration settings.
    pub mouse: MouseConfig,
    /// Logging settings.
    pub logging: crate::log::LogConfig,
}

impl Config {
    /// Clamps layout and border values to safe ranges and resolves
    /// theme colors for any unset border color fields.
    pub fn validate(&mut self) {
        self.resolve_borders();
        self.layout.gap = self.layout.gap.clamp(0, 200);
        self.layout.ratio = self.layout.ratio.clamp(0.1, 0.9);
        self.borders.width = self.borders.width.clamp(0, 32);
    }

    /// Resolves border colors: empty → theme default, named → theme hex.
    fn resolve_borders(&mut self) {
        let theme = self.theme.resolve();
        self.borders.focused = theme
            .resolve_color(&self.borders.focused, theme.border_focused())
            .to_string();
        self.borders.monocle = theme
            .resolve_color(&self.borders.monocle, theme.border_monocle())
            .to_string();
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
