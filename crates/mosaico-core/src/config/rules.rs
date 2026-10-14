/// Window rule types and evaluation logic.
///
/// Rules determine which windows Mosaico should manage (tile) and
/// which should be left floating. They are evaluated in order — the
/// first matching rule wins.
use serde::{Deserialize, Serialize};

use crate::{Error, Result};

use super::keybinding::{self, Keybinding};

/// A rule that determines whether a window should be managed (tiled).
///
/// Rules are evaluated in order. The first matching rule wins.
/// If no rule matches, the window is managed by default.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowRule {
    /// Match windows with this exact class name (case-insensitive).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_class: Option<String>,
    /// Match windows whose title contains this string (case-insensitive).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_title: Option<String>,
    /// Match windows whose executable path ends with this value
    /// (case-insensitive, on a path component boundary). Accepts a bare
    /// file name like `"app.exe"`, a partial path like
    /// `"Vendor\\app.exe"`, or a full path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_path: Option<String>,
    /// Whether matching windows should be managed (tiled).
    pub manage: bool,
}

/// Returns the default window rules (empty).
///
/// Rules are sourced from the community `rules.toml` (downloaded from
/// the mosaico-rules repository) and the user's `user-rules.toml`.
pub fn default_rules() -> Vec<WindowRule> {
    Vec::new()
}

/// Evaluates window rules to decide if a window should be managed.
///
/// Returns `true` if the window should be tiled. When no rule matches,
/// defaults to `true`.
///
/// `path` is the full path of the window's executable, or empty if it
/// could not be determined.
pub fn should_manage(class: &str, title: &str, path: &str, rules: &[WindowRule]) -> bool {
    matching_rule(class, title, path, rules).unwrap_or(true)
}

/// Returns the `manage` value of the first rule matching the window,
/// or `None` when no rule matches.
///
/// Used for windows that only get tiled when a rule explicitly opts
/// them in, such as app windows that draw their own title bar.
pub fn matching_rule(class: &str, title: &str, path: &str, rules: &[WindowRule]) -> Option<bool> {
    rules
        .iter()
        .find(|rule| matches_rule(class, title, path, rule))
        .map(|rule| rule.manage)
}

fn matches_rule(class: &str, title: &str, path: &str, rule: &WindowRule) -> bool {
    if let Some(ref mc) = rule.match_class
        && !class.eq_ignore_ascii_case(mc)
    {
        return false;
    }
    if let Some(ref mt) = rule.match_title {
        if mt.is_empty() {
            // match_title = "" means the window title must be empty.
            if !title.is_empty() {
                return false;
            }
        } else if !title
            .to_ascii_lowercase()
            .contains(&mt.to_ascii_lowercase())
        {
            return false;
        }
    }
    if let Some(ref mp) = rule.match_path
        && !path_ends_with(path, mp)
    {
        return false;
    }
    rule.match_class.is_some() || rule.match_title.is_some() || rule.match_path.is_some()
}

/// Case-insensitive check that `path` ends with `suffix` at a path
/// component boundary, treating `/` and `\` as equivalent.
///
/// `"app.exe"` matches `C:\Vendor\app.exe` but not `C:\Vendor\myapp.exe`.
fn path_ends_with(path: &str, suffix: &str) -> bool {
    let normalize = |s: &str| s.replace('/', "\\").to_ascii_lowercase();
    let path = normalize(path);
    let suffix = normalize(suffix);
    if suffix.is_empty() || !path.ends_with(&suffix) {
        return false;
    }
    let rest = &path[..path.len() - suffix.len()];
    rest.is_empty() || rest.ends_with('\\') || suffix.starts_with('\\')
}

/// Wrapper for deserializing the keybindings file.
///
/// The file contains a top-level `[[keybinding]]` array of tables.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct KeybindingsFile {
    #[serde(default = "keybinding::defaults")]
    pub(crate) keybinding: Vec<Keybinding>,
}

/// Wrapper for deserializing the rules file.
///
/// The file contains a top-level `[[rule]]` array of tables.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RulesFile {
    #[serde(default = "default_rules")]
    pub(crate) rule: Vec<WindowRule>,
}

/// Wrapper for deserializing the user rules file.
///
/// Unlike [`RulesFile`], an empty file results in zero rules
/// (no hardcoded defaults are injected).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct UserRulesFile {
    #[serde(default)]
    pub(crate) rule: Vec<WindowRule>,
}

/// Validates a TOML string as a rules file.
///
/// Returns the parsed rules or an [`Error::Config`] without a path, as
/// the content is not in a file yet. Used by the community-rules
/// downloader to verify content before caching.
///
/// # Errors
///
/// Returns `Err` if the TOML content is malformed or does not match
/// the expected rules schema.
pub fn validate_rules(content: &str) -> Result<Vec<WindowRule>> {
    let file: RulesFile =
        toml::from_str(content).map_err(|e| Error::config(None, e.to_string()))?;
    Ok(file.rule)
}
