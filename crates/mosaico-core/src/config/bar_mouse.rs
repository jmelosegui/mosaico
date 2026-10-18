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
}

impl Default for BarMouseConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            scroll: BarScroll::default(),
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

#[cfg(test)]
#[path = "bar_mouse_tests.rs"]
mod tests;
