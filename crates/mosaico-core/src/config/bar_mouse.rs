//! The `[mouse]` section of `bar.toml`: what the mouse does on the bar.

use serde::{Deserialize, Serialize};

/// What the mouse does on the bar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BarMouseConfig {
    /// Whether the bar takes clicks and the wheel. With `false` the bar
    /// lets the pointer through to the windows under it, as it always
    /// did before.
    pub enabled: bool,
    /// What scrolling over the bar does.
    pub scroll: BarScroll,
    /// What clicks on the widgets run, the `[[mouse.click]]` entries. Each
    /// replaces the binding of its widget and button, a default included,
    /// adds one, or with [`ClickBinding::NONE`] removes it, and of two for
    /// the same widget and button the later one wins. A file without any,
    /// or with `click = []`, has the defaults, [`ClickBinding::defaults`].
    pub click: Vec<ClickBinding>,
}

impl Default for BarMouseConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            scroll: BarScroll::default(),
            click: ClickBinding::defaults(),
        }
    }
}

/// What scrolling over the bar does.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BarScroll {
    /// Steps through the workspaces of the bar's monitor: up or left to
    /// the one before, down or right to the one after.
    #[default]
    Workspaces,
    /// Nothing.
    None,
}

/// An entry of `[[mouse.click]]`: the action a click of a button runs on
/// a kind of widget, as the file writes them, with `None` for a key it
/// leaves out.
///
/// The names are kept as text and checked as the bar takes the config
/// (see [`WidgetClicks`](crate::bar::WidgetClicks)), so an entry with a
/// name the bar does not know is left out on its own, and the rest of the
/// file applies.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ClickBinding {
    /// The kind of widget, by its `type`, such as `"layout"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub widget: Option<String>,
    /// The button: `"left"`, `"right"` or `"middle"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub button: Option<String>,
    /// The action, named as in `keybindings.toml`, such as
    /// `"cycle-layout"`, or [`NONE`](Self::NONE).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
}

impl ClickBinding {
    /// The `action` that removes the binding of its widget and button, a
    /// default included, as `scroll = "none"` turns the wheel off.
    pub const NONE: &'static str = "none";

    /// The binding of a click of `button` on the `widget` kind to `action`.
    pub fn new(widget: &str, button: &str, action: &str) -> Self {
        Self {
            widget: Some(widget.to_string()),
            button: Some(button.to_string()),
            action: Some(action.to_string()),
        }
    }

    /// The bindings a bar has before `[[mouse.click]]` changes any: on the
    /// layout widget the left button cycles the layout and the right one
    /// toggles monocle, and on the paused widget the left button resumes
    /// the hotkeys.
    pub fn defaults() -> Vec<Self> {
        vec![
            Self::new("layout", "left", "cycle-layout"),
            Self::new("layout", "right", "toggle-monocle"),
            Self::new("paused", "left", "toggle-pause"),
        ]
    }

    /// The bindings in effect with `entries` over the defaults: the
    /// entries in order, each replacing the binding of its widget and
    /// button, adding one, or with [`NONE`](Self::NONE) removing it. An
    /// entry that leaves out a key is skipped.
    pub fn effective(entries: &[Self]) -> Vec<Self> {
        let mut rows = Self::defaults();
        for entry in entries {
            let (Some(widget), Some(button), Some(action)) =
                (&entry.widget, &entry.button, &entry.action)
            else {
                continue;
            };
            let at = rows.iter().position(|r| {
                r.widget.as_ref() == Some(widget) && r.button.as_ref() == Some(button)
            });
            match (at, action == Self::NONE) {
                (Some(at), true) => {
                    rows.remove(at);
                }
                (Some(at), false) => rows[at] = entry.clone(),
                (None, false) => rows.push(entry.clone()),
                (None, true) => {}
            }
        }
        rows
    }

    /// The `[[mouse.click]]` entries that give exactly the bindings in
    /// `rows`: those that are not a default, and a
    /// [`NONE`](Self::NONE) entry for each default `rows` has no binding
    /// of its widget and button to replace. Of two rows for one widget
    /// and button the later counts. Empty when `rows` are the defaults.
    pub fn entries_for(rows: &[Self]) -> Vec<Self> {
        let key = |b: &Self| (b.widget.clone(), b.button.clone());
        let mut kept: Vec<&Self> = Vec::new();
        for row in rows {
            kept.retain(|k| key(k) != key(row));
            kept.push(row);
        }
        let defaults = Self::defaults();
        let mut entries: Vec<Self> = defaults
            .iter()
            .filter(|d| !kept.iter().any(|k| key(k) == key(d)))
            .map(|d| Self {
                action: Some(Self::NONE.to_string()),
                ..d.clone()
            })
            .collect();
        entries.extend(kept.into_iter().filter(|k| !defaults.contains(k)).cloned());
        entries
    }
}

#[cfg(test)]
#[path = "bar_mouse_tests.rs"]
mod tests;
