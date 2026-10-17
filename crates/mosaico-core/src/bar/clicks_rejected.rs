//! The `[[mouse.click]]` entries a bar leaves out, and why.

use std::fmt;

use crate::Action;
use crate::action::Scope;
use crate::config::{ClickBinding, WidgetKind};

use super::super::Button;

/// An entry of `[[mouse.click]]` the bar leaves out, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectedClick {
    /// The entry, as the file writes it.
    pub binding: ClickBinding,
    /// Why the bar leaves it out.
    pub problem: ClickProblem,
}

/// Why the bar leaves out an entry of `[[mouse.click]]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClickProblem {
    /// It has none of these keys: every key it leaves out, in the order
    /// `widget`, `button`, `action`.
    Missing(Vec<&'static str>),
    /// The value of this key is empty.
    Empty(&'static str),
    /// No widget has the type it names.
    UnknownWidget,
    /// Its button is not `left`, `right` or `middle`.
    UnknownButton,
    /// It names no action, for the reason given.
    UnknownAction(String),
    /// Its action acts on the focused window, which can be on another
    /// monitor than the bar clicked.
    FocusedWindow,
}

/// What `binding` has a click run, `None` for [`ClickBinding::NONE`],
/// which removes the binding, or why the bar cannot run it: every key it
/// leaves out, or else its first value, in the order `widget`, `button`,
/// `action`, that is empty or that the bar does not know.
pub(super) fn check(
    binding: &ClickBinding,
) -> Result<(WidgetKind, Button, Option<Action>), ClickProblem> {
    let keys = keys(binding);
    let missing = keys.iter().filter(|(_, value)| value.is_none());
    let missing: Vec<&'static str> = missing.map(|&(key, _)| key).collect();
    let [Some(widget), Some(button), Some(action)] = keys.map(|(_, value)| value) else {
        return Err(ClickProblem::Missing(missing));
    };
    let widget = filled("widget", widget)?;
    let kind = widget.parse().map_err(|_| ClickProblem::UnknownWidget)?;
    let button = filled("button", button)?;
    let button = button.parse().map_err(|_| ClickProblem::UnknownButton)?;
    let action = filled("action", action)?;
    if action == ClickBinding::NONE {
        return Ok((kind, button, None));
    }
    let action: Action = action.parse().map_err(ClickProblem::UnknownAction)?;
    if action.scope() == Scope::FocusedWindow {
        return Err(ClickProblem::FocusedWindow);
    }
    Ok((kind, button, Some(action)))
}

/// The keys of `binding` and their values, in the order the bar names
/// them in.
fn keys(binding: &ClickBinding) -> [(&'static str, Option<&str>); 3] {
    [
        ("widget", binding.widget.as_deref()),
        ("button", binding.button.as_deref()),
        ("action", binding.action.as_deref()),
    ]
}

/// `value`, the value of `key`, unless it is empty.
fn filled<'a>(key: &'static str, value: &'a str) -> Result<&'a str, ClickProblem> {
    if value.is_empty() {
        Err(ClickProblem::Empty(key))
    } else {
        Ok(value)
    }
}

/// `items` as a sentence lists them: "a", "a and b", "a, b and c".
fn and_list(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [only] => only.clone(),
        [first @ .., last] => format!("{} and {last}", first.join(", ")),
    }
}

impl fmt::Display for ClickProblem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing(keys) => {
                let nos: Vec<String> = keys.iter().map(|key| format!("no {key}")).collect();
                write!(f, "it has {}", and_list(&nos))
            }
            Self::Empty(key) => write!(f, "the {key} is empty"),
            Self::UnknownWidget => f.write_str("no widget has that type"),
            Self::UnknownButton => f.write_str("the button is not left, right or middle"),
            Self::UnknownAction(why) => f.write_str(why),
            Self::FocusedWindow => f.write_str(
                "the action is about the focused window, which can be on another monitor \
                 than the bar; a click may only run an action on a monitor or the whole daemon",
            ),
        }
    }
}

impl fmt::Display for RejectedClick {
    /// The entry as the file writes it, with the keys it has, and why the
    /// bar leaves it out.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let keys: Vec<String> = keys(&self.binding)
            .into_iter()
            .filter_map(|(key, value)| Some(format!("{key} = {:?}", value?)))
            .collect();
        let entry = if keys.is_empty() {
            "with no keys".to_string()
        } else {
            format!("with {}", and_list(&keys))
        };
        write!(f, "[[mouse.click]] {entry} is ignored: {}", self.problem)
    }
}
