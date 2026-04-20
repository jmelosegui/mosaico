use std::path::PathBuf;

use super::bar::BarConfig;
use super::keybinding;
use super::rules::{KeybindingsFile, RulesFile, UserRulesFile};
use super::{Config, Keybinding, WindowRule, default_rules};

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

/// Returns the user rules file path: `~/.config/mosaico/user-rules.toml`.
pub fn user_rules_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("user-rules.toml"))
}

/// Returns the bar config file path: `~/.config/mosaico/bar.toml`.
pub fn bar_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("bar.toml"))
}

/// Tries to load and parse `config.toml`.
///
/// Returns `Ok(Config)` on success, or an error string describing
/// what went wrong (IO error, parse error, etc.).
///
/// # Errors
///
/// Returns `Err` if the config path cannot be determined, the file
/// cannot be read, or the TOML content is invalid.
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
///
/// # Errors
///
/// Returns `Err` if the keybindings path cannot be determined, the file
/// cannot be read, or the TOML content is invalid.
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

/// Loads keybindings and appends any missing defaults to the user's file.
///
/// Compares the user's configured actions against the built-in defaults.
/// Any default action not already bound by the user is appended to the
/// keybindings file so new bindings from future versions are picked up
/// automatically, without overwriting anything the user has configured.
///
/// Falls back to `load_keybindings()` if the file cannot be read or written.
pub fn merge_missing_keybindings() -> Vec<Keybinding> {
    let path = match keybindings_path() {
        Some(p) if p.exists() => p,
        _ => return load_keybindings(),
    };

    let user = match try_load_keybindings() {
        Ok(kb) => kb,
        Err(e) => {
            eprintln!("Warning: {e}");
            return keybinding::defaults();
        }
    };

    let defaults = keybinding::defaults();
    let missing: Vec<&Keybinding> = defaults
        .iter()
        .filter(|d| !user.iter().any(|u| u.action == d.action))
        .collect();

    if missing.is_empty() {
        return user;
    }

    // Append missing bindings to the file.
    let mut addition =
        String::from("\n# Added automatically — new defaults from this version of mosaico\n");
    for kb in &missing {
        addition.push_str(&keybinding_toml_entry(kb));
    }

    match std::fs::OpenOptions::new().append(true).open(&path) {
        Ok(mut f) => {
            use std::io::Write;
            if let Err(e) = f.write_all(addition.as_bytes()) {
                eprintln!("Warning: could not append missing keybindings: {e}");
                return user;
            }
        }
        Err(e) => {
            eprintln!("Warning: could not open keybindings file for appending: {e}");
            return user;
        }
    }

    eprintln!(
        "Info: appended {} missing default keybinding(s) to keybindings.toml",
        missing.len()
    );

    // Return the full merged set.
    let mut merged = user;
    merged.extend(missing.into_iter().cloned());
    merged
}

/// Formats a single keybinding as a `[[keybinding]]` TOML entry.
fn keybinding_toml_entry(kb: &Keybinding) -> String {
    let modifiers: Vec<String> = kb
        .modifiers
        .iter()
        .map(|m| {
            let s = match m {
                keybinding::Modifier::Alt => "alt",
                keybinding::Modifier::Shift => "shift",
                keybinding::Modifier::Ctrl => "ctrl",
                keybinding::Modifier::Win => "win",
            };
            format!("\"{s}\"")
        })
        .collect();
    format!(
        "\n[[keybinding]]\naction = \"{}\"\nkey = \"{}\"\nmodifiers = [{}]\n",
        kb.action,
        kb.key,
        modifiers.join(", ")
    )
}

/// Loads bar config and appends any missing default widgets to the user's file.
///
/// Compares the user's `[[left]]` and `[[right]]` widget lists against the
/// built-in defaults by widget type. Any default widget type not already
/// present in the user's file is appended so new widgets from future versions
/// are picked up automatically, without overwriting anything the user has set.
///
/// Falls back to `load_bar()` if the file cannot be read or written.
pub fn merge_missing_bar_widgets() -> super::bar::BarConfig {
    use super::bar::{BarConfig, WidgetConfig};
    use serde::Serialize;

    let path = match bar_path() {
        Some(p) if p.exists() => p,
        _ => return load_bar(),
    };

    let mut user = match try_load_bar() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Warning: {e}");
            return BarConfig::default();
        }
    };

    let defaults = BarConfig::default();

    // Find widget types (by enum discriminant) present in defaults but missing
    // from the user's left/right lists. Users may intentionally remove widgets,
    // but a type that never existed before must be a new default.
    let missing_left: Vec<&WidgetConfig> = defaults
        .left
        .iter()
        .filter(|d| {
            !user
                .left
                .iter()
                .any(|u| std::mem::discriminant(u) == std::mem::discriminant(*d))
        })
        .collect();

    let missing_right: Vec<&WidgetConfig> = defaults
        .right
        .iter()
        .filter(|d| {
            !user
                .right
                .iter()
                .any(|u| std::mem::discriminant(u) == std::mem::discriminant(*d))
        })
        .collect();

    if missing_left.is_empty() && missing_right.is_empty() {
        return user;
    }

    // Serialize missing entries as TOML using a wrapper struct.
    #[derive(Serialize)]
    struct BarSideDiff<'a> {
        #[serde(skip_serializing_if = "Vec::is_empty")]
        left: Vec<&'a WidgetConfig>,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        right: Vec<&'a WidgetConfig>,
    }

    let diff = BarSideDiff {
        left: missing_left.clone(),
        right: missing_right.clone(),
    };

    let toml_addition = match toml::to_string(&diff) {
        Ok(s) => {
            format!("\n# Added automatically — new defaults from this version of mosaico\n{s}")
        }
        Err(e) => {
            eprintln!("Warning: could not serialize missing bar widgets: {e}");
            return user;
        }
    };

    match std::fs::OpenOptions::new().append(true).open(&path) {
        Ok(mut f) => {
            use std::io::Write;
            if let Err(e) = f.write_all(toml_addition.as_bytes()) {
                eprintln!("Warning: could not append missing bar widgets: {e}");
                return user;
            }
        }
        Err(e) => {
            eprintln!("Warning: could not open bar.toml for appending: {e}");
            return user;
        }
    }

    let total = missing_left.len() + missing_right.len();
    eprintln!("Info: appended {total} missing default bar widget(s) to bar.toml");

    // Return merged config.
    user.left.extend(missing_left.into_iter().cloned());
    user.right.extend(missing_right.into_iter().cloned());
    user
}

/// Tries to load and parse `rules.toml`.
///
/// Returns the parsed rules or an error string.
///
/// # Errors
///
/// Returns `Err` if the rules path cannot be determined, the file
/// cannot be read, or the TOML content is invalid.
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

/// Tries to load and parse `user-rules.toml`.
///
/// Returns the parsed rules or an error string.
///
/// # Errors
///
/// Returns `Err` if the user-rules path cannot be determined, the file
/// cannot be read, or the TOML content is invalid.
pub fn try_load_user_rules() -> Result<Vec<WindowRule>, String> {
    let path = user_rules_path().ok_or("could not determine user-rules path")?;
    let content = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let file: UserRulesFile =
        toml::from_str(&content).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(file.rule)
}

/// Loads user rules from `~/.config/mosaico/user-rules.toml`.
///
/// Returns an empty vec if the file is missing or invalid.
pub fn load_user_rules() -> Vec<WindowRule> {
    load_or_default(user_rules_path(), try_load_user_rules, Vec::new)
}

/// Loads and merges both rule sets: user rules first, then community rules.
///
/// User rules are prepended so they take priority (first match wins
/// in [`super::should_manage`]). Falls back gracefully if either file
/// is missing or invalid.
pub fn load_merged_rules() -> Vec<WindowRule> {
    let mut rules = load_user_rules();
    rules.extend(load_rules());
    rules
}

/// Tries to load and parse `bar.toml`.
///
/// Returns the parsed bar config or an error string. Colors are **not**
/// resolved here — the caller must call `resolve_colors(theme)` with
/// the global theme from `config.toml`.
///
/// # Errors
///
/// Returns `Err` if the bar config path cannot be determined, the file
/// cannot be read, or the TOML content is invalid.
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
