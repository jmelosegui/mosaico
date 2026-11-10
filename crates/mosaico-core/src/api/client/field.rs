//! One value of the state, by path, as a line of text.

use std::fmt;
use std::str::FromStr;

use serde_json::Value;

/// A path to one value of the state, such as `focus.window.title` or
/// `monitors.0.active_workspace`: keys of objects and indexes into lists,
/// joined by dots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldPath(Vec<String>);

impl FieldPath {
    /// The topic the path starts in, such as `focus`.
    pub fn topic(&self) -> &str {
        &self.0[0]
    }

    /// The value at this path in `state`, if there is one.
    pub fn lookup<'a>(&self, state: &'a Value) -> Option<&'a Value> {
        self.0.iter().try_fold(state, |value, step| match value {
            Value::Object(map) => map.get(step),
            Value::Array(items) => items.get(step.parse::<usize>().ok()?),
            _ => None,
        })
    }
}

impl FromStr for FieldPath {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let steps: Vec<String> = text.split('.').map(str::to_string).collect();
        if steps.iter().any(String::is_empty) {
            return Err(format!(
                "`{text}` is not a path such as `focus.window.title`"
            ));
        }
        Ok(Self(steps))
    }
}

impl fmt::Display for FieldPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0.join("."))
    }
}

/// `value` as one line of text: a string with its backslashes and line
/// breaks escaped as `\\`, `\n` and `\r`, so the line always ends where
/// the value does; a number or a boolean as written; nothing for a null or
/// a missing value; and a list or an object as compact JSON, which is one
/// line already.
pub fn field_text(value: Option<&Value>) -> String {
    let text = match value {
        None | Some(Value::Null) => return String::new(),
        Some(Value::String(text)) => text,
        Some(other) => return other.to_string(),
    };
    let mut escaped = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            c => escaped.push(c),
        }
    }
    escaped
}
