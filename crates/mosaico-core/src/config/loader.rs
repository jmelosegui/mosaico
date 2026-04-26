use std::path::{Path, PathBuf};

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
/// If a legacy `[layout.workspaces]` section is detected, the file is
/// backed up and rewritten in place to the new `[workspaces.layouts]`
/// shape before parsing.
///
/// # Errors
///
/// Returns `Err` if the config path cannot be determined, the file
/// cannot be read, or the TOML content is invalid.
pub fn try_load() -> Result<Config, String> {
    let path = config_path().ok_or("could not determine config path")?;
    let content = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;

    let content = match migrate_config_if_needed(&path, &content)? {
        Some(rewritten) => rewritten,
        None => content,
    };

    let mut config: Config =
        toml::from_str(&content).map_err(|e| format!("{}: {e}", path.display()))?;
    config.validate();
    Ok(config)
}

/// Detects and applies all legacy -> current schema migrations:
///
/// - `[layout.workspaces]` -> `[workspaces.layouts]`
/// - `[borders] focused = ...` / `monocle = ...` -> `[borders.colors]`
///
/// On detection of any legacy pattern: backs up the original file
/// (aborting if the backup write fails), rewrites the config in place
/// using `toml_edit` so comments and formatting survive, and returns
/// the new content for the caller to parse.
///
/// Returns `Ok(None)` when no migration was needed.
fn migrate_config_if_needed(path: &Path, content: &str) -> Result<Option<String>, String> {
    let mut doc: toml_edit::DocumentMut = content
        .parse()
        .map_err(|e| format!("{}: {e}", path.display()))?;

    let has_legacy_workspaces = doc
        .get("layout")
        .and_then(|t| t.as_table())
        .is_some_and(|t| t.contains_key("workspaces"));

    let has_legacy_borders = doc
        .get("borders")
        .and_then(|t| t.as_table())
        .is_some_and(|t| t.contains_key("focused") || t.contains_key("monocle"));

    if !has_legacy_workspaces && !has_legacy_borders {
        return Ok(None);
    }

    // Back up first -- abort migration if backup fails so we never mutate
    // without a safety net.
    let backup = backup_path(path);
    std::fs::copy(path, &backup).map_err(|e| {
        format!(
            "could not back up {} to {}: {e}",
            path.display(),
            backup.display()
        )
    })?;

    let mut applied: Vec<&'static str> = Vec::new();

    if has_legacy_workspaces {
        migrate_workspaces_section(&mut doc, path)?;
        applied.push("[layout.workspaces] -> [workspaces.layouts]");
    }

    if has_legacy_borders {
        migrate_border_colors(&mut doc, path)?;
        applied.push("[borders] focused/monocle -> [borders.colors]");
    }

    let new_content = doc.to_string();
    std::fs::write(path, &new_content)
        .map_err(|e| format!("could not write migrated config to {}: {e}", path.display()))?;

    eprintln!(
        "Info: migrated config keys ({}). Backup saved at {}",
        applied.join(", "),
        backup.display()
    );

    Ok(Some(new_content))
}

/// Moves `[layout.workspaces]` to `[workspaces.layouts]`.
///
/// If the user already has `[workspaces.layouts]`, keeps it and drops
/// the legacy mapping (the new section wins).
fn migrate_workspaces_section(doc: &mut toml_edit::DocumentMut, path: &Path) -> Result<(), String> {
    let legacy = doc
        .get_mut("layout")
        .and_then(|t| t.as_table_mut())
        .and_then(|t| t.remove("workspaces"));

    let Some(legacy_item) = legacy else {
        return Ok(());
    };

    if doc.get("workspaces").is_none() {
        doc.insert(
            "workspaces",
            toml_edit::Item::Table(toml_edit::Table::new()),
        );
    }
    let workspaces = doc
        .get_mut("workspaces")
        .and_then(|t| t.as_table_mut())
        .ok_or_else(|| {
            format!(
                "{}: [workspaces] is not a table -- cannot migrate",
                path.display()
            )
        })?;

    if !workspaces.contains_key("layouts") {
        workspaces.insert("layouts", legacy_item);
    }
    Ok(())
}

/// Moves top-level `focused` / `monocle` keys from `[borders]` into a
/// `[borders.colors]` sub-table.
///
/// If the user already has `[borders.colors].focused` or `.monocle`,
/// the existing nested values win and the legacy keys are dropped.
fn migrate_border_colors(doc: &mut toml_edit::DocumentMut, path: &Path) -> Result<(), String> {
    let borders = doc.get_mut("borders").and_then(|t| t.as_table_mut());
    let Some(borders) = borders else {
        return Ok(());
    };

    let legacy_focused = borders.remove("focused");
    let legacy_monocle = borders.remove("monocle");
    if legacy_focused.is_none() && legacy_monocle.is_none() {
        return Ok(());
    }

    if !borders.contains_key("colors") {
        borders.insert("colors", toml_edit::Item::Table(toml_edit::Table::new()));
    }
    let colors = borders
        .get_mut("colors")
        .and_then(|t| t.as_table_mut())
        .ok_or_else(|| {
            format!(
                "{}: [borders.colors] is not a table -- cannot migrate",
                path.display()
            )
        })?;

    if let Some(item) = legacy_focused
        && !colors.contains_key("focused")
    {
        colors.insert("focused", item);
    }
    if let Some(item) = legacy_monocle
        && !colors.contains_key("monocle")
    {
        colors.insert("monocle", item);
    }
    Ok(())
}

/// Returns a non-clobbering backup path next to `path`.
///
/// Uses `<stem>.pre-0.9.bak` as the base name so the user can tell what
/// the backup is for. Appends a numeric suffix if it already exists.
fn backup_path(path: &Path) -> PathBuf {
    let base = path.with_extension("toml.pre-0.9.bak");
    if !base.exists() {
        return base;
    }
    for n in 1..1000 {
        let candidate = path.with_extension(format!("toml.pre-0.9.bak.{n}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    base
}

/// Loads the configuration from disk, falling back to defaults.
///
/// After loading, values are clamped to safe ranges via [`Config::validate`].
/// Non-existent files silently return defaults; other IO errors are logged.
pub fn load() -> Config {
    load_or_default(config_path(), try_load, Config::default)
}

/// Snippet appended to `config.toml` when the `[workspaces]` section is
/// missing. Documents the new options so users notice them on first load
/// after upgrading.
const WORKSPACES_SECTION_SNIPPET: &str = r##"
# Added automatically -- new defaults from this version of mosaico
[workspaces]
# How a workspace switch is applied across monitors.
# "per-monitor" (default): only the focused monitor switches.
# "global": all monitors switch in lockstep, like Windows virtual desktops.
mode = "per-monitor"

# Per-workspace layout overrides (workspace number 1-8).
# Available layouts: "bsp", "vertical-stack", "three-column".
# [workspaces.layouts]
# 1 = "vertical-stack"
# 3 = "three-column"
"##;

/// Appends a documented `[workspaces]` section to `config.toml` when the
/// user's file is missing it.
///
/// Mirrors [`merge_missing_keybindings`] and [`merge_missing_bar_widgets`]:
/// new top-level sections introduced by future versions of mosaico are
/// appended as commented or default-valued blocks so users discover them
/// without losing their existing customizations.
///
/// No-op when the file does not exist (a brand-new install will get the
/// full section from `mosaico init`'s template). Errors are logged and
/// swallowed -- failure here must not prevent the daemon from starting.
pub fn merge_missing_config_sections() {
    let path = match config_path() {
        Some(p) if p.exists() => p,
        _ => return,
    };

    match append_missing_workspaces_section(&path) {
        Ok(true) => eprintln!(
            "Info: appended [workspaces] section to {} (new in this version)",
            path.display()
        ),
        Ok(false) => {}
        Err(e) => eprintln!("Warning: {e}"),
    }

    match append_missing_unfocused_border_hint(&path) {
        Ok(true) => eprintln!(
            "Info: appended unfocused border hint to {} (new in this version)",
            path.display()
        ),
        Ok(false) => {}
        Err(e) => eprintln!("Warning: {e}"),
    }
}

/// Comment block appended to `config.toml` when the file has no
/// reference to the `unfocused` border color, so existing users
/// discover the new option without having to read release notes.
const UNFOCUSED_BORDER_HINT_SNIPPET: &str = r##"
# Added automatically -- new in this version of mosaico
# The `unfocused` color under [borders.colors] sets the border drawn
# around tiled windows that do not currently have focus. Recognized
# values:
#   ""           use the active theme's muted gray default (Mocha:
#                #6c7086). This is the default when the field is
#                absent.
#   "none"       disable unfocused borders entirely; only the focused
#                window will get a border.
#   "#RRGGBB"    explicit hex color.
#   "blue"       any named theme color (mauve, teal, lavender, etc.).
#
# Example:
#   [borders.colors]
#   unfocused = "none"
"##;

/// Path-pure helper for [`merge_missing_config_sections`]: returns
/// `Ok(true)` when an append happened, `Ok(false)` when the file already
/// has a `[workspaces]` section.
fn append_missing_workspaces_section(path: &Path) -> Result<bool, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("could not read {}: {e}", path.display()))?;

    let doc: toml_edit::DocumentMut = content
        .parse()
        .map_err(|e| format!("could not parse {}: {e}", path.display()))?;

    if doc.contains_key("workspaces") {
        return Ok(false);
    }

    // Make sure the snippet starts on a fresh line even if the existing
    // file does not end in a newline.
    let mut to_append = String::new();
    if !content.ends_with('\n') {
        to_append.push('\n');
    }
    to_append.push_str(WORKSPACES_SECTION_SNIPPET);

    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .append(true)
        .open(path)
        .map_err(|e| format!("could not open {} for appending: {e}", path.display()))?;
    f.write_all(to_append.as_bytes())
        .map_err(|e| format!("could not append to {}: {e}", path.display()))?;
    Ok(true)
}

/// Adds a documented `unfocused` field to the user's config so the
/// new option is discoverable.
///
/// Two strategies, in order:
/// 1. If `[borders.colors]` is an active table, insert `unfocused = ""`
///    into it with a leading comment. Empty string preserves current
///    behavior (theme-default muted gray).
/// 2. Otherwise (no active table, or the table is commented out), if
///    the file does not already mention `unfocused`, append a
///    standalone comment block at the end of the file.
///
/// Both paths are idempotent: a second run finds the key (or the
/// substring) already present and does nothing.
fn append_missing_unfocused_border_hint(path: &Path) -> Result<bool, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("could not read {}: {e}", path.display()))?;

    if let Some(updated) = insert_unfocused_into_active_section(&content)? {
        std::fs::write(path, updated)
            .map_err(|e| format!("could not write {}: {e}", path.display()))?;
        return Ok(true);
    }

    // Fall back to the comment-only hint when there is no active
    // [borders.colors] section to extend. Skip when any reference to
    // `unfocused` already exists (active key, comment, prior hint).
    if content.contains("unfocused") {
        return Ok(false);
    }

    let mut to_append = String::new();
    if !content.ends_with('\n') {
        to_append.push('\n');
    }
    to_append.push_str(UNFOCUSED_BORDER_HINT_SNIPPET);

    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .append(true)
        .open(path)
        .map_err(|e| format!("could not open {} for appending: {e}", path.display()))?;
    f.write_all(to_append.as_bytes())
        .map_err(|e| format!("could not append to {}: {e}", path.display()))?;
    Ok(true)
}

/// Inline comment placed above the inserted `unfocused` key. Kept on
/// its own constant so the tests can pin the wording without growing
/// the inline path.
const UNFOCUSED_INLINE_PREFIX: &str = "\n\
    # Color drawn around unfocused tiled windows.\n\
    # \"none\" disables unfocused borders; \"\" (empty) uses the\n\
    # theme default (muted gray); a hex code or named theme color\n\
    # works too.\n";

/// If `content` has an active `[borders.colors]` table that is missing
/// the `unfocused` key, returns the rewritten file contents with the
/// key inserted. Returns `Ok(None)` when no edit is needed.
fn insert_unfocused_into_active_section(content: &str) -> Result<Option<String>, String> {
    let mut doc: toml_edit::DocumentMut =
        content.parse().map_err(|e| format!("parse error: {e}"))?;

    let Some(colors) = doc
        .get_mut("borders")
        .and_then(|t| t.as_table_mut())
        .and_then(|t| t.get_mut("colors"))
        .and_then(|t| t.as_table_mut())
    else {
        return Ok(None);
    };

    if colors.contains_key("unfocused") {
        return Ok(None);
    }

    colors.insert("unfocused", toml_edit::value(""));
    if let Some(mut key) = colors.key_mut("unfocused") {
        key.leaf_decor_mut()
            .set_prefix(toml_edit::RawString::from(UNFOCUSED_INLINE_PREFIX));
    }

    Ok(Some(doc.to_string()))
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

#[cfg(test)]
mod migration_tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    /// Returns a unique temp path for each call so parallel tests do not
    /// collide on the shared system temp directory.
    fn unique_temp_path() -> PathBuf {
        let mut p = std::env::temp_dir();
        let pid = std::process::id();
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        p.push(format!("mosaico-migration-test-{pid}-{n}.toml"));
        p
    }

    fn cleanup(path: &Path) {
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(path.with_extension("toml.pre-0.9.bak"));
        for n in 1..10 {
            let _ = std::fs::remove_file(path.with_extension(format!("toml.pre-0.9.bak.{n}")));
        }
    }

    #[test]
    fn migrates_legacy_section_and_writes_backup() {
        let path = unique_temp_path();
        let original = "\
[layout]
gap = 8

[layout.workspaces]
1 = \"vertical-stack\"
3 = \"three-column\"
";
        std::fs::write(&path, original).unwrap();

        let migrated = migrate_config_if_needed(&path, original).unwrap();

        assert!(migrated.is_some(), "expected migration to run");
        let new_content = migrated.unwrap();
        assert!(new_content.contains("[workspaces.layouts]"));
        assert!(!new_content.contains("[layout.workspaces]"));
        assert!(new_content.contains("1 = \"vertical-stack\""));

        let backup = path.with_extension("toml.pre-0.9.bak");
        assert!(backup.exists(), "backup file should exist");
        let backup_contents = std::fs::read_to_string(&backup).unwrap();
        assert_eq!(backup_contents, original, "backup must match original");

        let on_disk = std::fs::read_to_string(&path).unwrap();
        assert_eq!(
            on_disk, new_content,
            "rewritten file must match returned content"
        );

        cleanup(&path);
    }

    #[test]
    fn no_legacy_section_is_a_noop() {
        let path = unique_temp_path();
        let original = "\
[layout]
gap = 8

[workspaces]
mode = \"global\"

[workspaces.layouts]
2 = \"bsp\"
";
        std::fs::write(&path, original).unwrap();

        let migrated = migrate_config_if_needed(&path, original).unwrap();
        assert!(
            migrated.is_none(),
            "should not migrate when no legacy section"
        );

        let backup = path.with_extension("toml.pre-0.9.bak");
        assert!(!backup.exists(), "no backup should be created");

        let on_disk = std::fs::read_to_string(&path).unwrap();
        assert_eq!(on_disk, original, "file must be untouched");

        cleanup(&path);
    }

    #[test]
    fn second_run_is_idempotent() {
        let path = unique_temp_path();
        let original = "\
[layout]
gap = 8

[layout.workspaces]
1 = \"bsp\"
";
        std::fs::write(&path, original).unwrap();

        // First pass migrates.
        let first = migrate_config_if_needed(&path, original).unwrap();
        assert!(first.is_some());
        let after_first = std::fs::read_to_string(&path).unwrap();

        // Second pass sees the already-migrated file and does nothing.
        let second = migrate_config_if_needed(&path, &after_first).unwrap();
        assert!(second.is_none(), "second migration must be a no-op");

        let collision_backup = path.with_extension("toml.pre-0.9.bak.1");
        assert!(
            !collision_backup.exists(),
            "no second backup should be created"
        );

        cleanup(&path);
    }

    #[test]
    fn existing_workspaces_layouts_wins_over_legacy() {
        let path = unique_temp_path();
        let original = "\
[layout]
gap = 8

[layout.workspaces]
1 = \"bsp\"

[workspaces]
mode = \"global\"

[workspaces.layouts]
2 = \"three-column\"
";
        std::fs::write(&path, original).unwrap();

        let migrated = migrate_config_if_needed(&path, original).unwrap().unwrap();

        // Legacy section is gone.
        assert!(!migrated.contains("[layout.workspaces]"));
        // The pre-existing [workspaces.layouts] is preserved (workspace 2).
        assert!(migrated.contains("2 = \"three-column\""));
        // The legacy mapping (workspace 1 = bsp) is dropped because the
        // user already had a [workspaces.layouts] section.
        assert!(!migrated.contains("1 = \"bsp\""));

        cleanup(&path);
    }

    #[test]
    fn appends_workspaces_section_when_missing() {
        let path = unique_temp_path();
        let original = "\
[layout]
gap = 8

[borders]
width = 4
";
        std::fs::write(&path, original).unwrap();

        let appended = append_missing_workspaces_section(&path).unwrap();
        assert!(appended, "should append when [workspaces] is missing");

        let on_disk = std::fs::read_to_string(&path).unwrap();
        assert!(
            on_disk.starts_with(original),
            "existing content must be preserved verbatim"
        );
        assert!(on_disk.contains("[workspaces]"));
        assert!(on_disk.contains("mode = \"per-monitor\""));
        assert!(on_disk.contains("# [workspaces.layouts]"));

        cleanup(&path);
    }

    #[test]
    fn does_not_append_when_section_present() {
        let path = unique_temp_path();
        let original = "\
[layout]
gap = 8

[workspaces]
mode = \"global\"
";
        std::fs::write(&path, original).unwrap();

        let appended = append_missing_workspaces_section(&path).unwrap();
        assert!(!appended, "should not append when section already present");

        let on_disk = std::fs::read_to_string(&path).unwrap();
        assert_eq!(on_disk, original, "file must be untouched");

        cleanup(&path);
    }

    #[test]
    fn append_handles_file_without_trailing_newline() {
        let path = unique_temp_path();
        let original = "[layout]\ngap = 8"; // no trailing newline
        std::fs::write(&path, original).unwrap();

        append_missing_workspaces_section(&path).unwrap();

        let on_disk = std::fs::read_to_string(&path).unwrap();
        // The section header must land on its own line, not be glued to
        // `gap = 8`.
        assert!(on_disk.contains("gap = 8\n"));
        assert!(on_disk.contains("[workspaces]"));

        cleanup(&path);
    }

    #[test]
    fn unfocused_inserted_inline_when_borders_colors_active() {
        let path = unique_temp_path();
        // Active [borders.colors] with focused/monocle but no unfocused.
        let original = "\
[borders]
width = 4

[borders.colors]
focused = \"blue\"
monocle = \"red\"
";
        std::fs::write(&path, original).unwrap();

        let updated = append_missing_unfocused_border_hint(&path).unwrap();
        assert!(updated, "should insert unfocused when section is active");

        let on_disk = std::fs::read_to_string(&path).unwrap();

        // Round-trips through serde with the new field set to "".
        let cfg: super::Config = toml::from_str(&on_disk).unwrap();
        assert_eq!(cfg.borders.colors.focused, "blue");
        assert_eq!(cfg.borders.colors.monocle, "red");
        assert_eq!(cfg.borders.colors.unfocused, "");

        // The key sits inside [borders.colors] alongside focused and
        // monocle, not as an orphan block at the end of the file.
        let colors_idx = on_disk.find("[borders.colors]").unwrap();
        let unfocused_idx = on_disk.find("unfocused").unwrap();
        assert!(unfocused_idx > colors_idx);

        // The leading comment block survived the rewrite.
        assert!(on_disk.contains("Color drawn around unfocused tiled windows."));
        // No fallback bottom hint was added.
        assert!(!on_disk.contains("Added automatically"));

        cleanup(&path);
    }

    #[test]
    fn unfocused_inline_insertion_is_idempotent() {
        let path = unique_temp_path();
        let original = "\
[borders.colors]
focused = \"blue\"
";
        std::fs::write(&path, original).unwrap();

        assert!(append_missing_unfocused_border_hint(&path).unwrap());
        let after_first = std::fs::read_to_string(&path).unwrap();

        assert!(!append_missing_unfocused_border_hint(&path).unwrap());
        let after_second = std::fs::read_to_string(&path).unwrap();
        assert_eq!(after_first, after_second);

        cleanup(&path);
    }

    #[test]
    fn appends_unfocused_border_hint_when_section_missing() {
        let path = unique_temp_path();
        // No [borders.colors] section at all; the section in the file
        // is commented out.
        let original = "\
[borders]
width = 4
corner_style = \"small\"
# [borders.colors]
# focused = \"blue\"
";
        std::fs::write(&path, original).unwrap();

        let appended = append_missing_unfocused_border_hint(&path).unwrap();
        assert!(appended, "should append bottom hint when no active section");

        let on_disk = std::fs::read_to_string(&path).unwrap();
        assert!(on_disk.starts_with(original), "existing content preserved");
        assert!(on_disk.contains("unfocused"));
        assert!(on_disk.contains("\"none\""));
        // Existing [borders.colors] is still commented out.
        assert!(!on_disk.contains("\nfocused = \"blue\""));

        cleanup(&path);
    }

    #[test]
    fn unfocused_bottom_hint_is_idempotent() {
        let path = unique_temp_path();
        let original = "[borders]\nwidth = 4\n";
        std::fs::write(&path, original).unwrap();

        assert!(append_missing_unfocused_border_hint(&path).unwrap());
        let after_first = std::fs::read_to_string(&path).unwrap();

        assert!(!append_missing_unfocused_border_hint(&path).unwrap());
        let after_second = std::fs::read_to_string(&path).unwrap();
        assert_eq!(after_first, after_second);

        cleanup(&path);
    }

    #[test]
    fn unfocused_hint_not_appended_when_user_already_set_it() {
        let path = unique_temp_path();
        let original = "\
[borders.colors]
unfocused = \"#deadbe\"
";
        std::fs::write(&path, original).unwrap();

        let appended = append_missing_unfocused_border_hint(&path).unwrap();
        assert!(
            !appended,
            "active 'unfocused' key should suppress all updates"
        );

        let on_disk = std::fs::read_to_string(&path).unwrap();
        assert_eq!(on_disk, original, "file untouched when user already set it");

        cleanup(&path);
    }

    #[test]
    fn unfocused_hint_not_appended_when_template_already_mentions_it() {
        let path = unique_temp_path();
        // Mirrors the new commented template generated by `mosaico init`.
        let original = "\
[borders]
width = 4

# [borders.colors]
# focused = \"blue\"
# monocle = \"green\"
# unfocused = \"none\"
";
        std::fs::write(&path, original).unwrap();

        let appended = append_missing_unfocused_border_hint(&path).unwrap();
        assert!(
            !appended,
            "commented template mentioning 'unfocused' should suppress the hint"
        );

        cleanup(&path);
    }

    #[test]
    fn backup_path_avoids_collision() {
        let path = unique_temp_path();
        std::fs::write(&path, "").unwrap();

        let first = backup_path(&path);
        assert_eq!(first.extension().and_then(|e| e.to_str()), Some("bak"));
        std::fs::write(&first, "").unwrap();

        let second = backup_path(&path);
        assert_ne!(first, second);
        assert!(
            second.to_string_lossy().ends_with(".bak.1"),
            "second backup should append .1 suffix, got {}",
            second.display()
        );

        cleanup(&path);
    }

    #[test]
    fn migrates_legacy_border_colors() {
        let path = unique_temp_path();
        let original = "\
[borders]
width = 4
focused = \"#ff0000\"
monocle = \"#00ff00\"
";
        std::fs::write(&path, original).unwrap();

        let migrated = migrate_config_if_needed(&path, original)
            .unwrap()
            .expect("migration should run");

        // Re-parse and assert the values landed in the new shape.
        let cfg: super::Config = toml::from_str(&migrated).unwrap();
        assert_eq!(cfg.borders.width, 4);
        assert_eq!(cfg.borders.colors.focused, "#ff0000");
        assert_eq!(cfg.borders.colors.monocle, "#00ff00");

        // Backup preserved.
        let backup = path.with_extension("toml.pre-0.9.bak");
        assert!(backup.exists());
        assert_eq!(std::fs::read_to_string(&backup).unwrap(), original);

        cleanup(&path);
    }

    #[test]
    fn border_migration_is_idempotent() {
        let path = unique_temp_path();
        let original = "\
[borders]
width = 4
focused = \"blue\"
";
        std::fs::write(&path, original).unwrap();

        migrate_config_if_needed(&path, original).unwrap().unwrap();
        let after_first = std::fs::read_to_string(&path).unwrap();
        let second = migrate_config_if_needed(&path, &after_first).unwrap();
        assert!(second.is_none(), "second border migration must be a no-op");

        cleanup(&path);
    }

    #[test]
    fn existing_border_colors_wins_over_legacy() {
        let path = unique_temp_path();
        let original = "\
[borders]
width = 4
focused = \"red\"
monocle = \"green\"

[borders.colors]
focused = \"blue\"
";
        std::fs::write(&path, original).unwrap();

        let migrated = migrate_config_if_needed(&path, original).unwrap().unwrap();
        let cfg: super::Config = toml::from_str(&migrated).unwrap();

        // Existing nested focused wins over legacy.
        assert_eq!(cfg.borders.colors.focused, "blue");
        // Legacy monocle migrates because no nested value existed.
        assert_eq!(cfg.borders.colors.monocle, "green");

        cleanup(&path);
    }

    #[test]
    fn workspace_and_border_migrations_run_together() {
        let path = unique_temp_path();
        let original = "\
[layout]
gap = 8

[layout.workspaces]
1 = \"vertical-stack\"

[borders]
width = 4
focused = \"#abcdef\"
";
        std::fs::write(&path, original).unwrap();

        let migrated = migrate_config_if_needed(&path, original).unwrap().unwrap();

        // Both legacy patterns are gone after migration.
        assert!(!migrated.contains("[layout.workspaces]"));
        assert!(migrated.contains("[workspaces.layouts]"));
        assert!(migrated.contains("[borders.colors]"));
        // Top-level `focused` under [borders] should not be there anymore;
        // the only remaining `focused = ...` line lives under [borders.colors].
        let bare_focused_lines = migrated
            .lines()
            .filter(|l| l.trim_start().starts_with("focused = "))
            .count();
        assert_eq!(
            bare_focused_lines, 1,
            "exactly one focused = line, under [borders.colors]"
        );

        // Only one backup created for the combined run.
        let backup = path.with_extension("toml.pre-0.9.bak");
        let collision = path.with_extension("toml.pre-0.9.bak.1");
        assert!(backup.exists());
        assert!(!collision.exists(), "single backup, not one per migration");

        cleanup(&path);
    }
}
