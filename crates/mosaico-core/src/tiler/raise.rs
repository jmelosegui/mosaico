//! Raising a window the user asked for by its application: a launch
//! binding with `focus` set brings an open window into view instead of
//! starting the program again.

use crate::WindowId;

use super::{Plan, Tiler};

impl Tiler {
    /// Every window the tiler manages, tiled or floating, on every monitor
    /// and workspace, in that order: monitor by monitor, workspace by
    /// workspace, the tiles in layout order, then the floating windows.
    pub(super) fn managed_windows(&self) -> Vec<WindowId> {
        self.monitors
            .iter()
            .flat_map(|mon| &mon.workspaces)
            .flat_map(|ws| ws.windows().iter().chain(ws.floating()))
            .copied()
            .collect()
    }

    /// Focuses `window` and brings it into view: its monitor switches to
    /// its workspace when that is out of view, as when the user picks it
    /// from the taskbar, and a monocle window in front of it steps back.
    ///
    /// Does nothing for a window the tiler does not manage.
    pub(super) fn raise(&mut self, plan: &mut Plan, window: WindowId) {
        let Some((mon_idx, ws_idx)) = self.find_managed(window) else {
            return;
        };
        self.focused_monitor = mon_idx;
        if ws_idx != self.monitors[mon_idx].active_workspace {
            let Ok(n) = u8::try_from(ws_idx + 1) else {
                return;
            };
            self.goto_workspace(plan, n);
        }
        self.focus_window(plan, window, false);
        let ws = self.monitors[mon_idx].active_ws();
        if ws.monocle() && ws.monocle_window() != Some(window) {
            self.exit_monocle(plan, mon_idx);
        }
    }
}

/// The window to raise among `matching`, which are in the order of
/// [`Tiler::managed_windows`]: the one after `focused` when that is one of
/// them, wrapping around, so pressing the key again cycles through the
/// application's windows; otherwise the first.
pub(super) fn next_match(matching: &[WindowId], focused: Option<WindowId>) -> Option<WindowId> {
    let current = focused.and_then(|focused| matching.iter().position(|w| *w == focused));
    match current {
        Some(at) => matching.get((at + 1) % matching.len()).copied(),
        None => matching.first().copied(),
    }
}
