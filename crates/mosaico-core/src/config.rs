use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::Action;

/// Top-level configuration for Mosaico.
///
/// Loaded from `~/.config/mosaico/config.toml`. Missing sections
/// fall back to defaults thanks to `#[serde(default)]`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Layout algorithm parameters.
    pub layout: LayoutConfig,
    /// Global keybindings.
    pub keybindings: Vec<Keybinding>,
}

/// Layout algorithm settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LayoutConfig {
    /// Gap in pixels between windows and screen edges.
    pub gap: i32,
    /// Ratio of space given to the first window in each split (0.0–1.0).
    pub ratio: f64,
}

/// A user-configured keybinding that maps a key combination to an action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybinding {
    /// The action to trigger.
    pub action: Action,
    /// Key name (e.g. "J", "Enter", "Space", "F1").
    pub key: String,
    /// Modifier keys (e.g. ["alt", "shift"]).
    pub modifiers: Vec<Modifier>,
}

/// Keyboard modifier keys.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Modifier {
    Alt,
    Shift,
    Ctrl,
    Win,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            layout: LayoutConfig::default(),
            keybindings: default_keybindings(),
        }
    }
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self { gap: 8, ratio: 0.5 }
    }
}

/// Returns the config directory: `~/.config/mosaico/`.
pub fn config_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".config").join("mosaico"))
}

/// Returns the config file path: `~/.config/mosaico/config.toml`.
pub fn config_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("config.toml"))
}

/// Loads the configuration from disk, falling back to defaults.
///
/// If the file doesn't exist, returns defaults silently.
/// If the file exists but can't be parsed, logs a warning and returns defaults.
pub fn load() -> Config {
    let Some(path) = config_path() else {
        return Config::default();
    };

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Config::default(),
    };

    match toml::from_str(&content) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Warning: failed to parse {}: {e}", path.display());
            Config::default()
        }
    }
}

/// Default keybindings matching the Phase 8 hardcoded bindings.
fn default_keybindings() -> Vec<Keybinding> {
    use Modifier::{Alt, Ctrl, Shift};

    vec![
        Keybinding {
            action: Action::FocusNext,
            key: "J".into(),
            modifiers: vec![Alt, Shift],
        },
        Keybinding {
            action: Action::FocusPrev,
            key: "K".into(),
            modifiers: vec![Alt, Shift],
        },
        Keybinding {
            action: Action::SwapNext,
            key: "Enter".into(),
            modifiers: vec![Alt, Shift],
        },
        Keybinding {
            action: Action::SwapPrev,
            key: "Enter".into(),
            modifiers: vec![Alt, Ctrl],
        },
        Keybinding {
            action: Action::Retile,
            key: "R".into(),
            modifiers: vec![Alt, Shift],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_five_keybindings() {
        // Arrange / Act
        let config = Config::default();

        // Assert
        assert_eq!(config.keybindings.len(), 5);
        assert_eq!(config.layout.gap, 8);
        assert_eq!(config.layout.ratio, 0.5);
    }

    #[test]
    fn partial_toml_uses_defaults_for_missing_sections() {
        // Arrange
        let toml_str = "[layout]\ngap = 16\n";

        // Act
        let config: Config = toml::from_str(toml_str).unwrap();

        // Assert
        assert_eq!(config.layout.gap, 16);
        assert_eq!(config.layout.ratio, 0.5); // default
        assert_eq!(config.keybindings.len(), 5); // defaults
    }

    #[test]
    fn keybinding_roundtrips_through_toml() {
        // Arrange
        let config = Config::default();

        // Act
        let serialized = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();

        // Assert
        assert_eq!(deserialized.keybindings.len(), config.keybindings.len());
        assert_eq!(deserialized.layout.gap, config.layout.gap);
    }
}
