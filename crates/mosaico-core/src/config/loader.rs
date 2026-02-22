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

/// Tries to load and parse `config.toml`.
///
/// Returns `Ok(Config)` on success, or an error string describing
/// what went wrong (IO error, parse error, etc.).
pub fn try_load() -> Result<Config, String> {
    let path = config_path().ok_or("could not determine config path")?;
    let content = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut config: Config =
        toml::from_str(&content).map_err(|e| format!("{}: {e}", path.display()))?;
    config.validate();
    Ok(config)
}

/// Loads the configuration from disk, falling back to defaults.
///
/// After loading, values are clamped to safe ranges via [`Config::validate`].
/// Non-existent files silently return defaults; other IO errors are logged.
pub fn load() -> Config {
    load_or_default(try_load, Config::default)
}

/// Tries to load and parse `keybindings.toml`.
///
/// Returns the parsed keybindings or an error string.
pub fn try_load_keybindings() -> Result<Vec<Keybinding>, String> {
    let path = keybindings_path().ok_or("could not determine keybindings path")?;
    let content = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let file: KeybindingsFile =
        toml::from_str(&content).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(file.keybinding)
}

/// Loads keybindings from `~/.config/mosaico/keybindings.toml`.
///
/// Falls back to the built-in defaults if the file is missing or invalid.
pub fn load_keybindings() -> Vec<Keybinding> {
    load_or_default(try_load_keybindings, keybinding::defaults)
}

/// Tries to load and parse `rules.toml`.
///
/// Returns the parsed rules or an error string.
pub fn try_load_rules() -> Result<Vec<WindowRule>, String> {
    let path = rules_path().ok_or("could not determine rules path")?;
    let content = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let file: RulesFile =
        toml::from_str(&content).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(file.rule)
}

/// Loads window rules from `~/.config/mosaico/rules.toml`.
///
/// Falls back to the built-in defaults if the file is missing or invalid.
pub fn load_rules() -> Vec<WindowRule> {
    load_or_default(try_load_rules, default_rules)
}

/// Loads a config value from disk, falling back to defaults.
///
/// Non-existent files silently return defaults; other IO errors are logged.
fn load_or_default<T>(try_load: impl FnOnce() -> Result<T, String>, default: impl Fn() -> T) -> T {
    match try_load() {
        Ok(val) => val,
        Err(e) if is_file_not_found(&e) => default(),
        Err(e) => {
            eprintln!("Warning: {e}");
            default()
        }
    }
}

/// Returns true if the error message indicates a missing file.
fn is_file_not_found(e: &str) -> bool {
    e.contains("cannot find the path") || e.contains("The system cannot find")
}
