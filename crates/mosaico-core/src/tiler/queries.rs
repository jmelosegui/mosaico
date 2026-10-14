//! What the tiler can tell about its state.

use crate::WindowId;

use super::{MonitorState, Tiler};

impl Tiler {
    /// Every monitor, in the order of the displays handed to
    /// [`Tiler::new`] or in the latest
    /// [`Input::Displays`](super::Input::Displays): left to right when
    /// they come from [`Enumerator::monitors`](super::Enumerator::monitors).
    pub fn monitors(&self) -> &[MonitorState] {
        &self.monitors
    }

    /// Index of the monitor the user is working on.
    pub fn focused_monitor(&self) -> usize {
        self.focused_monitor
    }

    /// The window that has focus, as far as the tiler is concerned.
    pub fn focused_window(&self) -> Option<WindowId> {
        self.focused_window
    }

    /// Total number of managed windows.
    pub fn window_count(&self) -> usize {
        self.monitors
            .iter()
            .flat_map(|m| &m.workspaces)
            .map(|ws| ws.len())
            .sum()
    }
}
