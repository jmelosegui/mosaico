//! What one monitor's bar shows of the tiler and the daemon.

use crate::action::MAX_WORKSPACES;
use crate::{LayoutKind, WindowId};

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
