use std::path::PathBuf;

use super::bar::BarConfig;
use super::keybinding;
use super::{Config, Keybinding, KeybindingsFile, RulesFile, WindowRule, default_rules};

/// Returns the config directory: `~/.config/mosaico/`.
pub fn config_dir() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE").map(|h| PathBuf::from(h).join(".config").join("mosaico"))
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

/// Returns the bar config file path: `~/.config/mosaico/bar.toml`.
pub fn bar_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("bar.toml"))
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
    load_or_default(config_path(), try_load, Config::default)
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
    load_or_default(
        keybindings_path(),
        try_load_keybindings,
        keybinding::defaults,
    )
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
    load_or_default(rules_path(), try_load_rules, default_rules)
}

/// Tries to load and parse `bar.toml`.
///
/// Returns the parsed bar config or an error string. Colors are **not**
/// resolved here — the caller must call `resolve_colors(theme)` with
/// the global theme from `config.toml`.
pub fn try_load_bar() -> Result<BarConfig, String> {
    let path = bar_path().ok_or("could not determine bar config path")?;
    let content = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut config: BarConfig =
        toml::from_str(&content).map_err(|e| format!("{}: {e}", path.display()))?;
    config.validate();
    Ok(config)
}

/// Loads the bar configuration from disk, falling back to defaults.
///
/// Non-existent files silently return defaults; other IO errors are logged.
pub fn load_bar() -> BarConfig {
    load_or_default(bar_path(), try_load_bar, BarConfig::default)
}

/// Loads a config value from disk, falling back to defaults.
///
/// Non-existent files silently return defaults; other IO errors are logged.
fn load_or_default<T>(
    path: Option<PathBuf>,
    try_load: impl FnOnce() -> Result<T, String>,
    default: impl Fn() -> T,
) -> T {
    match path {
        Some(p) if !p.exists() => default(),
        None => default(),
        _ => match try_load() {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Warning: {e}");
                default()
            }
        },
    }
}
