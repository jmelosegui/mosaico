//! What one monitor's bar shows of the tiler and the daemon.

use crate::action::MAX_WORKSPACES;
use crate::api::State;
use crate::{LayoutKind, StableMonitorId, WindowId};

/// What one monitor's bar shows of the tiler and the daemon, and what the
/// pointer on it needs to know of them.
///
/// The system widgets show [`SystemReadings`](super::SystemReadings)
/// instead, which are the same on every monitor.
#[derive(Debug, Clone, PartialEq)]
pub struct BarState {
    /// Index of the workspace on screen, from 0.
    pub active_workspace: usize,
    /// How many workspaces the monitor has.
    pub workspace_count: usize,
    /// Layout of the workspace on screen.
    pub layout: LayoutKind,
    /// Whether the workspace on screen is in monocle mode, showing one
    /// window over the whole area.
    pub monocle: bool,
    /// The focused window. Only the monitor the user is working on has
    /// one.
    pub focused_window: Option<WindowId>,
    /// Notice of a newer version, such as "v0.10.0 available". Empty when
    /// there is none.
    pub update_text: String,
    /// Whether the hotkeys are paused.
    pub paused: bool,
    /// Whether stepping through the workspaces goes round from the last
    /// to the first, as `[workspaces] wrap` says, for the wheel.
    pub wrap: bool,
    /// Whether a window on screen on the monitor is maximized or
    /// fullscreen, under the bar, which then lets the pointer through.
    pub maximized: bool,
}

/// What a bar shows for a monitor the tiler has no state for.
impl Default for BarState {
    fn default() -> Self {
        Self {
            active_workspace: 0,
            workspace_count: usize::from(MAX_WORKSPACES),
            layout: LayoutKind::default(),
            monocle: false,
            focused_window: None,
            update_text: String::new(),
            paused: false,
            wrap: false,
            maximized: false,
        }
    }
}

impl BarState {
    /// What the bar of monitor `id` shows of `state`, or `None` when
    /// `state` has no such monitor.
    ///
    /// Only the monitor the user works on shows the focused window. A
    /// layout this build does not know shows as the default one.
    pub fn from_state(state: &State, id: &StableMonitorId) -> Option<Self> {
        let monitor = state.monitor(id)?;
        let active = monitor.active_workspace.saturating_sub(1) as usize;
        let workspace = monitor.workspaces.get(active);
        let focused_here = state.focus.monitor.as_ref() == Some(id);
        Some(Self {
            active_workspace: active,
            workspace_count: monitor.workspaces.len(),
            layout: (workspace.and_then(|ws| LayoutKind::from_key(&ws.layout))).unwrap_or_default(),
            monocle: workspace.is_some_and(|ws| ws.monocle),
            focused_window: (state.focus.window.as_ref())
                .filter(|_| focused_here)
                .map(|w| w.id),
            update_text: state.daemon.update.clone().unwrap_or_default(),
            paused: state.daemon.paused,
            wrap: state.daemon.wrap,
            maximized: monitor.maximized,
        })
    }
}

#[cfg(test)]
#[path = "tests/state_tests.rs"]
mod tests;
