/// Window rule types and evaluation logic.
///
/// Rules decide which windows Mosaico tiles, which it floats and which
/// it leaves alone. They are evaluated in order: the first matching rule
/// wins.
use serde::{Deserialize, Serialize};

use crate::{Error, Result};

use super::keybinding::{self, Keybinding};
use super::rule_action::{RawRule, RuleAction};

/// A rule that decides what happens to the windows it matches.
///
/// Rules are evaluated in order. The first matching rule wins. See
/// [`decide`](super::decide) for windows no rule matches.
///
/// A rules file writes what the rule does as `action = "tile"`,
/// `"float"` or `"ignore"`, or as the older `manage = true` or `false`,
/// which mean `"tile"` and `"ignore"`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "RawRule")]
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
    /// What happens to matching windows.
    pub action: RuleAction,
}

/// Returns the default window rules (empty).
///
/// Rules are sourced from the community `rules.toml` (downloaded from
/// the mosaico-rules repository) and the user's `user-rules.toml`.
pub fn default_rules() -> Vec<WindowRule> {
    Vec::new()
}

/// The first rule matching the window, or `None` when no rule matches.
pub fn matching_rule<'a>(
    class: &str,
    title: &str,
    path: &str,
    rules: &'a [WindowRule],
) -> Option<&'a WindowRule> {
    rules
        .iter()
        .find(|rule| matches_rule(class, title, path, rule))
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
