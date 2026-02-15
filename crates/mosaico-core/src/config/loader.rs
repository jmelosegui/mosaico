use std::path::PathBuf;

use super::keybinding;
use super::{Config, Keybinding, KeybindingsFile, RulesFile, WindowRule, default_rules};

/// Returns the config directory: `~/.config/mosaico/`.
pub fn config_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".config").join("mosaico"))
}

/// Returns the config file path: `~/.config/mosaico/config.toml`.
pub fn config_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("config.toml"))
}

/// Returns the keybindings file path: `~/.config/mosaico/keybindings.toml`.
pub fn keybindings_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("keybindings.toml"))
}

/// Returns the rules file path: `~/.config/mosaico/rules.toml`.
pub fn rules_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("rules.toml"))
}

/// Loads the configuration from disk, falling back to defaults.
///
/// After loading, values are clamped to safe ranges via [`Config::validate`].
/// Non-existent files silently return defaults; other IO errors are logged.
pub fn load() -> Config {
    let Some(path) = config_path() else {
        return Config::default();
    };
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Config::default(),
        Err(e) => {
            eprintln!("Warning: could not read {}: {e}", path.display());
            return Config::default();
        }
    };
    let mut config: Config = match toml::from_str(&content) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: failed to parse {}: {e}", path.display());
            Config::default()
        }
    };
    config.validate();
    config
}

/// Loads keybindings from `~/.config/mosaico/keybindings.toml`.
///
/// Falls back to the built-in defaults if the file is missing or invalid.
/// Non-existent files silently return defaults; other IO errors are logged.
pub fn load_keybindings() -> Vec<Keybinding> {
    let Some(path) = keybindings_path() else {
        return keybinding::defaults();
    };
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return keybinding::defaults(),
        Err(e) => {
            eprintln!("Warning: could not read {}: {e}", path.display());
            return keybinding::defaults();
        }
    };
    match toml::from_str::<KeybindingsFile>(&content) {
        Ok(file) => file.keybinding,
        Err(e) => {
            eprintln!("Warning: failed to parse {}: {e}", path.display());
            keybinding::defaults()
        }
    }
}

/// Loads window rules from `~/.config/mosaico/rules.toml`.
///
/// Falls back to the built-in defaults if the file is missing or invalid.
/// Non-existent files silently return defaults; other IO errors are logged.
pub fn load_rules() -> Vec<WindowRule> {
    let Some(path) = rules_path() else {
        return default_rules();
    };
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return default_rules(),
        Err(e) => {
            eprintln!("Warning: could not read {}: {e}", path.display());
            return default_rules();
        }
    };
    match toml::from_str::<RulesFile>(&content) {
        Ok(file) => file.rule,
        Err(e) => {
            eprintln!("Warning: failed to parse {}: {e}", path.display());
            default_rules()
        }
    }
}
