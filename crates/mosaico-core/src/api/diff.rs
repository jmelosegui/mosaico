//! The difference between two states, and applying it.

use serde::{Deserialize, Serialize};

use crate::StableMonitorId;

use super::{Daemon, Focus, Monitor, State, Theme};

/// What changed between two states: each topic that changed, in full, and
/// each monitor that changed or came, in full, by id.
///
/// Applying one is replacing objects by id, so a bar keeps no model of its
/// own: `older.apply(older.diff(&newer))` is `newer`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
#[serde(default)]
pub struct Update {
    /// The daemon, when it changed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub daemon: Option<Daemon>,
    /// The theme, when it changed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<Theme>,
    /// Each monitor that changed or was connected, in full.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub monitors: Vec<Monitor>,
    /// Where the user works, when it changed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus: Option<Focus>,
    /// What is gone.
    #[serde(skip_serializing_if = "Removed::is_empty")]
    pub removed: Removed,
}

/// What an [`Update`] takes away.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
#[serde(default)]
pub struct Removed {
    /// The ids of the monitors disconnected.
    #[cfg_attr(test, schemars(with = "Vec<String>"))]
    pub monitors: Vec<StableMonitorId>,
}

impl Removed {
    /// Whether nothing is taken away.
    pub fn is_empty(&self) -> bool {
        self.monitors.is_empty()
    }
}

impl Update {
    /// Whether nothing changed.
    pub fn is_empty(&self) -> bool {
        self.daemon.is_none()
            && self.theme.is_none()
            && self.monitors.is_empty()
            && self.focus.is_none()
            && self.removed.is_empty()
    }
}

impl State {
    /// What changed from this state to `newer`.
    pub fn diff(&self, newer: &State) -> Update {
        let changed = |old: &Monitor| self.monitor(&old.id) != Some(old);
        Update {
            daemon: (self.daemon != newer.daemon).then(|| newer.daemon.clone()),
            theme: (self.theme != newer.theme).then(|| newer.theme.clone()),
            monitors: newer
                .monitors
                .iter()
                .filter(|m| changed(m))
                .cloned()
                .collect(),
            focus: (self.focus != newer.focus).then(|| newer.focus.clone()),
            removed: Removed {
                monitors: (self.monitors.iter())
                    .filter(|m| newer.monitor(&m.id).is_none())
                    .map(|m| m.id.clone())
                    .collect(),
            },
        }
    }

    /// Takes in `update`: replaces each topic and monitor it carries, by
    /// id, drops the monitors it removes, and puts the monitors in
    /// `index` order again.
    pub fn apply(&mut self, update: Update) {
        if let Some(daemon) = update.daemon {
            self.daemon = daemon;
        }
        if let Some(theme) = update.theme {
            self.theme = theme;
        }
        if let Some(focus) = update.focus {
            self.focus = focus;
        }
        (self.monitors).retain(|m| !update.removed.monitors.contains(&m.id));
        for monitor in update.monitors {
            match self.monitors.iter_mut().find(|m| m.id == monitor.id) {
                Some(slot) => *slot = monitor,
                None => self.monitors.push(monitor),
            }
        }
        self.monitors.sort_by_key(|m| m.index);
    }
}
