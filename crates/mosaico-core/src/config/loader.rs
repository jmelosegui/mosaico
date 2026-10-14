use std::fmt;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;

use crate::{Error, Instance, Result};

use super::bar::BarConfig;
use super::keybinding;
use super::report;
use super::rules::{KeybindingsFile, RulesFile, UserRulesFile};
use super::{Config, Keybinding, WindowRule, default_rules};

/// The config files of one instance, all kept in one directory.
///
/// Every operation reads and writes files in [`dir`](Self::dir) and
/// nowhere else, so a store on a temporary directory exercises loading,
/// merging and migrating without touching the user's files. The daemon
/// and the CLI use [`ConfigStore::for_instance`].
///
/// A missing file is never created: loading it gives the defaults and
/// merging into it does nothing. Two kinds of operation change files
/// that exist: loading `config.toml` rewrites keys from older versions
/// after saving a backup next to it, and the `merge_missing_*` methods
/// add what newer versions introduced, appending it or, for the
/// `unfocused` border color, inserting it into `[borders.colors]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigStore {
    dir: PathBuf,
}

impl ConfigStore {
    /// A store for the config files in `dir`.
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    /// The store of `instance`, on the files in its config directory.
    pub fn for_instance(instance: &Instance) -> Self {
        Self::new(instance.config_dir())
    }

    /// The directory holding the files.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// `config.toml`: theme, layout, workspaces, borders, mouse and logging.
    pub fn config_path(&self) -> PathBuf {
        self.dir.join("config.toml")
    }

    /// `keybindings.toml`: the hotkeys.
    pub fn keybindings_path(&self) -> PathBuf {
        self.dir.join("keybindings.toml")
    }

    /// `rules.toml`: the community window rules the daemon downloads.
    pub fn rules_path(&self) -> PathBuf {
        self.dir.join("rules.toml")
    }

    /// `user-rules.toml`: the user's own window rules.
    pub fn user_rules_path(&self) -> PathBuf {
        self.dir.join("user-rules.toml")
    }

    /// `bar.toml`: the status bar.
    pub fn bar_path(&self) -> PathBuf {
        self.dir.join("bar.toml")
    }
}

impl ConfigStore {
    /// Loads and parses `config.toml`, then clamps its values with
    /// [`Config::validate`].
    ///
    /// If the file still has keys from older versions
    /// (`[layout.workspaces]`, or `focused` / `monocle` directly under
    /// `[borders]`), it is backed up and rewritten in place to the
    /// current shape before parsing.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Config`] naming the file if it cannot be
    /// read, cannot be migrated, or is not valid TOML for a [`Config`].
    pub fn try_load_config(&self) -> Result<Config> {
        let path = self.config_path();
        let content = read_file(&path)?;

        let content = match migrate_config_if_needed(&path, &content)? {
            Some(rewritten) => rewritten,
            None => content,
        };

        let mut config: Config = parse_toml(&path, &content)?;
        config.validate();
        Ok(config)
    }
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
fn migrate_config_if_needed(path: &Path, content: &str) -> Result<Option<String>> {
    let mut doc: toml_edit::DocumentMut = content.parse().map_err(|e| file_error(path, e))?;

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
        file_error(
            path,
            format_args!("could not back up to {}: {e}", backup.display()),
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
    std::fs::write(path, &new_content).map_err(|e| {
        file_error(
            path,
            format_args!("could not write the migrated config: {e}"),
        )
    })?;

    report::info(format_args!(
        "migrated config keys ({}). Backup saved at {}",
        applied.join(", "),
        backup.display()
    ));

    Ok(Some(new_content))
}

/// Moves `[layout.workspaces]` to `[workspaces.layouts]`.
///
/// If the user already has `[workspaces.layouts]`, keeps it and drops
/// the legacy mapping (the new section wins).
fn migrate_workspaces_section(doc: &mut toml_edit::DocumentMut, path: &Path) -> Result<()> {
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
        .ok_or_else(|| file_error(path, "[workspaces] is not a table -- cannot migrate"))?;

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
fn migrate_border_colors(doc: &mut toml_edit::DocumentMut, path: &Path) -> Result<()> {
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
        .ok_or_else(|| file_error(path, "[borders.colors] is not a table -- cannot migrate"))?;

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

impl ConfigStore {
    /// Loads `config.toml` like [`try_load_config`](Self::try_load_config),
    /// falling back to the defaults.
    ///
    /// A missing file silently gives the defaults; any other failure is
    /// reported as a warning, on standard error and in the log.
    pub fn load_config(&self) -> Config {
        load_or_default(
            &self.config_path(),
            || self.try_load_config(),
            Config::default,
        )
    }
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

impl ConfigStore {
    /// Adds what newer versions of mosaico introduced to `config.toml`:
    ///
    /// - A documented `[workspaces]` section, appended when the file has
    ///   none.
    /// - The `unfocused` border color. When `[borders.colors]` is an
    ///   active table without the key, `unfocused = ""` is inserted into
    ///   it under a comment, which rewrites the file in place without a
    ///   backup. Otherwise a comment block about it is appended, unless
    ///   the file already mentions `unfocused` anywhere.
    ///
    /// Mirrors [`merge_missing_keybindings`](Self::merge_missing_keybindings)
    /// and [`merge_missing_bar_widgets`](Self::merge_missing_bar_widgets):
    /// what future versions of mosaico introduce is added as commented or
    /// default-valued entries so users discover it without losing their
    /// existing customizations.
    ///
    /// No-op when the file does not exist (a brand-new install will get the
    /// full section from `mosaico init`'s template). Errors are logged and
    /// swallowed -- failure here must not prevent the daemon from starting.
    /// Only the daemon calls this, so it reports to the log alone.
    pub fn merge_missing_config_sections(&self) {
        let path = self.config_path();
        if !path.exists() {
            return;
        }

        match append_missing_workspaces_section(&path) {
            Ok(true) => crate::log_info!(
                "Appended [workspaces] section to {} (new in this version)",
                path.display()
            ),
            Ok(false) => {}
            Err(e) => crate::log_warn!("Could not add the [workspaces] section: {e}"),
        }

        match append_missing_unfocused_border_hint(&path) {
            Ok(true) => crate::log_info!(
                "Appended unfocused border hint to {} (new in this version)",
                path.display()
            ),
            Ok(false) => {}
            Err(e) => crate::log_warn!("Could not add the unfocused border hint: {e}"),
        }
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

/// Path-pure helper for [`ConfigStore::merge_missing_config_sections`]: returns
/// `Ok(true)` when an append happened, `Ok(false)` when the file already
/// has a `[workspaces]` section.
fn append_missing_workspaces_section(path: &Path) -> Result<bool> {
    let content = read_file(path)?;

    let doc: toml_edit::DocumentMut = content.parse().map_err(|e| file_error(path, e))?;

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

    append_to_file(path, &to_append)?;
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
fn append_missing_unfocused_border_hint(path: &Path) -> Result<bool> {
    let content = read_file(path)?;

    let inserted =
        insert_unfocused_into_active_section(&content).map_err(|e| file_error(path, e))?;
    if let Some(updated) = inserted {
        std::fs::write(path, updated)
            .map_err(|e| file_error(path, format_args!("could not write it: {e}")))?;
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

    append_to_file(path, &to_append)?;
    Ok(true)
}

/// Appends `text` to the config file at `path`.
fn append_to_file(path: &Path, text: &str) -> Result<()> {
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .append(true)
        .open(path)
        .map_err(|e| file_error(path, format_args!("could not open it for appending: {e}")))?;
    f.write_all(text.as_bytes())
        .map_err(|e| file_error(path, format_args!("could not append to it: {e}")))
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
fn insert_unfocused_into_active_section(
    content: &str,
) -> Result<Option<String>, toml_edit::TomlError> {
    let mut doc: toml_edit::DocumentMut = content.parse()?;

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

impl ConfigStore {
    /// Loads and parses `keybindings.toml`.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Config`] naming the file if it cannot be
    /// read or is not valid TOML for a keybindings file.
    pub fn try_load_keybindings(&self) -> Result<Vec<Keybinding>> {
        let file: KeybindingsFile = load_toml(&self.keybindings_path())?;
        Ok(file.keybinding)
    }

    /// Loads `keybindings.toml`, falling back to the built-in defaults.
    ///
    /// A missing file silently gives the defaults; any other failure is
    /// reported as a warning, on standard error and in the log.
    pub fn load_keybindings(&self) -> Vec<Keybinding> {
        load_or_default(
            &self.keybindings_path(),
            || self.try_load_keybindings(),
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
    /// Without a file, returns the defaults and creates nothing. When the
    /// file cannot be parsed, returns the defaults and leaves it alone.
    /// When appending fails, returns what the file has.
    /// Only the daemon calls this, so it reports to the log alone.
    pub fn merge_missing_keybindings(&self) -> Vec<Keybinding> {
        let path = self.keybindings_path();
        if !path.exists() {
            return self.load_keybindings();
        }

        let user = match self.try_load_keybindings() {
            Ok(kb) => kb,
            Err(e) => {
                crate::log_warn!("Using the default keybindings: {e}");
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
                    crate::log_warn!("Could not append missing keybindings: {e}");
                    return user;
                }
            }
            Err(e) => {
                crate::log_warn!("Could not open keybindings file for appending: {e}");
                return user;
            }
        }

        crate::log_info!(
            "Appended {} missing default keybinding(s) to keybindings.toml",
            missing.len()
        );

        // Return the full merged set.
        let mut merged = user;
        merged.extend(missing.into_iter().cloned());
        merged
    }
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

impl ConfigStore {
    /// Loads bar config and appends any missing default widgets to the user's file.
    ///
    /// Compares the user's `[[left]]` and `[[right]]` widget lists against the
    /// built-in defaults by widget type. Any default widget type not already
    /// present in the user's file is appended so new widgets from future versions
    /// are picked up automatically, without overwriting anything the user has set.
    ///
    /// Without a file, returns the defaults and creates nothing. When the
    /// file cannot be parsed, returns the defaults and leaves it alone.
    /// When appending fails, returns what the file has.
    /// Only the daemon calls this, so it reports to the log alone.
    pub fn merge_missing_bar_widgets(&self) -> BarConfig {
        use super::bar::WidgetConfig;
        use serde::Serialize;

        let path = self.bar_path();
        if !path.exists() {
            return self.load_bar();
        }

        let mut user = match self.try_load_bar() {
            Ok(cfg) => cfg,
            Err(e) => {
                crate::log_warn!("Using the default bar: {e}");
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
                crate::log_warn!("Could not serialize missing bar widgets: {e}");
                return user;
            }
        };

        match std::fs::OpenOptions::new().append(true).open(&path) {
            Ok(mut f) => {
                use std::io::Write;
                if let Err(e) = f.write_all(toml_addition.as_bytes()) {
                    crate::log_warn!("Could not append missing bar widgets: {e}");
                    return user;
                }
            }
            Err(e) => {
                crate::log_warn!("Could not open bar.toml for appending: {e}");
                return user;
            }
        }

        let total = missing_left.len() + missing_right.len();
        crate::log_info!("Appended {total} missing default bar widget(s) to bar.toml");

        // Return merged config.
        user.left.extend(missing_left.into_iter().cloned());
        user.right.extend(missing_right.into_iter().cloned());
        user
    }
}

impl ConfigStore {
    /// Loads and parses `rules.toml`.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Config`] naming the file if it cannot be
    /// read or is not valid TOML for a rules file.
    pub fn try_load_rules(&self) -> Result<Vec<WindowRule>> {
        let file: RulesFile = load_toml(&self.rules_path())?;
        Ok(file.rule)
    }

    /// Loads `rules.toml`, falling back to the built-in defaults.
    ///
    /// A missing file silently gives the defaults; any other failure is
    /// reported as a warning, on standard error and in the log.
    pub fn load_rules(&self) -> Vec<WindowRule> {
        load_or_default(&self.rules_path(), || self.try_load_rules(), default_rules)
    }

    /// Loads and parses `user-rules.toml`.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Config`] naming the file if it cannot be
    /// read or is not valid TOML for a rules file.
    pub fn try_load_user_rules(&self) -> Result<Vec<WindowRule>> {
        let file: UserRulesFile = load_toml(&self.user_rules_path())?;
        Ok(file.rule)
    }

    /// Loads `user-rules.toml`, falling back to no rules.
    ///
    /// A missing file silently gives no rules; any other failure is
    /// reported as a warning, on standard error and in the log.
    pub fn load_user_rules(&self) -> Vec<WindowRule> {
        load_or_default(
            &self.user_rules_path(),
            || self.try_load_user_rules(),
            Vec::new,
        )
    }

    /// Loads and merges both rule sets: user rules first, then community rules.
    ///
    /// User rules are prepended so they take priority (first match wins
    /// in [`should_manage`](super::should_manage)). Falls back gracefully
    /// if either file is missing or invalid.
    pub fn load_merged_rules(&self) -> Vec<WindowRule> {
        let mut rules = self.load_user_rules();
        rules.extend(self.load_rules());
        rules
    }

    /// Loads and parses `bar.toml`, then clamps its values with
    /// [`BarConfig::validate`].
    ///
    /// Colors are **not** resolved here; the caller must call
    /// `resolve_colors(theme)` with the global theme from `config.toml`.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Config`] naming the file if it cannot be
    /// read or is not valid TOML for a [`BarConfig`].
    pub fn try_load_bar(&self) -> Result<BarConfig> {
        let mut config: BarConfig = load_toml(&self.bar_path())?;
        config.validate();
        Ok(config)
    }

    /// Loads `bar.toml`, falling back to the defaults.
    ///
    /// A missing file silently gives the defaults; any other failure is
    /// reported as a warning, on standard error and in the log.
    pub fn load_bar(&self) -> BarConfig {
        load_or_default(&self.bar_path(), || self.try_load_bar(), BarConfig::default)
    }
}

/// Loads the file at `path` with `try_load`, falling back to `default`.
///
/// A missing file silently gives the default; any other failure is
/// reported as a warning, on standard error and in the log.
fn load_or_default<T>(
    path: &Path,
    try_load: impl FnOnce() -> Result<T>,
    default: impl Fn() -> T,
) -> T {
    if !path.exists() {
        return default();
    }
    match try_load() {
        Ok(val) => val,
        Err(e) => {
            report::warning(format_args!("{e}"));
            default()
        }
    }
}

/// Reads and parses the config file at `path`.
fn load_toml<T: DeserializeOwned>(path: &Path) -> Result<T> {
    parse_toml(path, &read_file(path)?)
}

/// Reads the config file at `path`, naming it in the error.
fn read_file(path: &Path) -> Result<String> {
    std::fs::read_to_string(path).map_err(|e| file_error(path, e))
}

/// Parses `content`, the TOML of the config file at `path`, naming the
/// file in the error.
fn parse_toml<T: DeserializeOwned>(path: &Path, content: &str) -> Result<T> {
    toml::from_str(content).map_err(|e| file_error(path, e))
}

/// A config error about the file at `path`. It reads
/// `<path>: <problem>`, as these errors always have.
fn file_error(path: &Path, problem: impl fmt::Display) -> Error {
    Error::config(Some(path.to_path_buf()), problem.to_string())
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

#[cfg(test)]
#[path = "loader_tests.rs"]
mod tests;
