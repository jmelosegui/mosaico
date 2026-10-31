//! What a window rule does with the windows it matches.

use serde::{Deserialize, Serialize};

use super::rules::WindowRule;

/// What happens to the windows a rule matches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleAction {
    /// Managed and tiled into the layout.
    Tile,
    /// Managed, but floating above the tiles of its workspace.
    Float,
    /// Never managed.
    Ignore,
}

/// A rule as written in a rules file: `action`, or the older `manage`.
#[derive(Deserialize)]
pub(super) struct RawRule {
    match_class: Option<String>,
    match_title: Option<String>,
    match_path: Option<String>,
    action: Option<RuleAction>,
    manage: Option<bool>,
}

impl TryFrom<RawRule> for WindowRule {
    type Error = String;

    fn try_from(raw: RawRule) -> Result<Self, Self::Error> {
        let action = match (raw.action, raw.manage) {
            (Some(action), None) => action,
            (None, Some(true)) => RuleAction::Tile,
            (None, Some(false)) => RuleAction::Ignore,
            (Some(_), Some(_)) => return Err("a rule takes `action` or `manage`, not both".into()),
            (None, None) => return Err("a rule needs an `action`".into()),
        };
        Ok(WindowRule {
            match_class: raw.match_class,
            match_title: raw.match_title,
            match_path: raw.match_path,
            action,
        })
    }
}

/// What the rules do with a window, from the first rule that matches it.
///
/// A window with a standard title bar is tiled when no rule matches. One
/// without, which looks like a splash screen or a popup, is only managed
/// when a rule asks to tile or float it.
///
/// `path` is the full path of the window's executable, or empty if it
/// could not be determined.
pub fn decide(
    class: &str,
    title: &str,
    path: &str,
    has_caption: bool,
    rules: &[WindowRule],
) -> RuleAction {
    let action = super::matching_rule(class, title, path, rules).map(|rule| rule.action);
    match action {
        Some(action) => action,
        None if has_caption => RuleAction::Tile,
        None => RuleAction::Ignore,
    }
}

#[cfg(test)]
#[path = "rule_action_tests.rs"]
mod tests;
