//! The `[workspaces]` section: how workspace switches apply across
//! monitors, how stepping through workspaces ends, and per-workspace
//! layouts.

use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize};

use crate::action::MAX_WORKSPACES;
use crate::layout::LayoutKind;

use super::report;

/// Workspace-level configuration.
///
/// Holds the workspace switching mode, whether stepping through the
/// workspaces wraps around, and per-workspace layout overrides. See
/// [`WorkspaceMode`] for the available modes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct WorkspacesConfig {
    /// How a workspace switch is applied across monitors.
    pub mode: WorkspaceMode,
    /// Whether stepping past the last workspace, with
    /// `goto-workspace-next` or the wheel on the bar, goes back to the
    /// first, and stepping before the first goes to the last. Off by
    /// default: a step stops at either end.
    pub wrap: bool,
    /// Per-workspace layout overrides (workspace number 1-8 to layout).
    ///
    /// TOML keys are always strings, so we deserialize via a helper that
    /// parses each key as a `u8` and silently drops keys outside the
    /// valid `1..=MAX_WORKSPACES` range (logging a warning for each).
    /// This lets users write `[workspaces.layouts]\n1 = "vertical-stack"`
    /// while the internal lookup remains `u8`-keyed.
    #[serde(deserialize_with = "deserialize_layouts_map")]
    pub layouts: HashMap<u8, LayoutKind>,
}

/// Deserializes a TOML table whose string keys represent workspace
/// numbers (`1`..=`MAX_WORKSPACES`). Invalid keys are dropped with a
/// warning so a typo on one workspace never fails the whole config.
fn deserialize_layouts_map<'de, D>(deserializer: D) -> Result<HashMap<u8, LayoutKind>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw: HashMap<String, LayoutKind> = HashMap::deserialize(deserializer)?;
    let mut out = HashMap::with_capacity(raw.len());
    for (key, value) in raw {
        match key.parse::<u8>() {
            Ok(n) if (1..=MAX_WORKSPACES).contains(&n) => {
                out.insert(n, value);
            }
            Ok(n) => {
                report::warning(format_args!(
                    "ignoring [workspaces.layouts] entry for workspace {n}: \
                     must be in 1..={MAX_WORKSPACES}"
                ));
            }
            Err(_) => {
                report::warning(format_args!(
                    "ignoring [workspaces.layouts] entry with non-numeric key {key:?}"
                ));
            }
        }
    }
    Ok(out)
}

/// How a workspace switch is applied across monitors.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum WorkspaceMode {
    /// Switching workspaces only affects the focused monitor (default).
    #[default]
    PerMonitor,
    /// Switching workspaces affects every monitor in lockstep, mirroring
    /// how Windows virtual desktops work.
    Global,
}
