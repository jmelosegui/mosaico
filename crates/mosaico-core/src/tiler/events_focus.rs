//! The focus event, the one the OS sends most often on its own.

use crate::WindowId;

use super::{Plan, Tiler};

impl Tiler {
    /// Follows a focus change the OS reported.
    ///
    /// Echoes of our own focus changes have already been filtered out
    /// by [`Expectations`](super::Expectations), so this is either the
    /// user picking a window or the OS handing focus on by itself.
    pub(super) fn on_focused(&mut self, plan: &mut Plan, window: WindowId) {
        if let Some((mon_idx, ws_idx)) = self.find_window(window) {
            if ws_idx != self.monitors[mon_idx].active_workspace {
                // The window is on a workspace that is not on screen,
                // which happens when the user picks it from the taskbar.
                // Switch there so it becomes visible.
                self.focused_monitor = mon_idx;
                self.goto_workspace(plan, (ws_idx + 1) as u8);
                self.focus_window(plan, window, true);
                let ws = self.monitors[mon_idx].active_ws();
                if ws.monocle() && ws.monocle_window() != Some(window) {
                    self.exit_monocle(plan, mon_idx);
                }
                return;
            }
            // The focused monitor may be parked on an empty workspace,
            // where nothing can hold focus, so the OS hands it back to
            // a window on another monitor. Following that would send the
            // next new window to the monitor the user just left.
            //
            // This comes after the inactive workspace case on purpose:
            // above it, picking a hidden window on another monitor from
            // the taskbar would never switch to its workspace and leave
            // it focused but invisible.
            if !self.focus_event_allowed(plan, mon_idx) {
                crate::log_debug!(
                    "focus-suppressed {} on mon {} (mon {} is parked on an empty workspace)",
                    window,
                    mon_idx,
                    self.focused_monitor
                );
                return;
            }
            crate::log_debug!("focus {} on mon {}", window, mon_idx);
            self.focused_window = Some(window);
            self.focused_monitor = mon_idx;
            self.focused_maximized = plan.is_maximized(window);
            let ws = self.monitors[mon_idx].active_ws();
            if ws.monocle() && ws.monocle_window() != Some(window) {
                self.exit_monocle(plan, mon_idx);
            } else {
                self.update_border(plan);
            }
        } else if let Some(owner) = plan.owner(window)
            && let Some(idx) = self.owning_monitor(owner)
        {
            // A dialog or property sheet of a managed window got focus.
            // Put the border on its owner so the user can see which
            // application it belongs to.
            self.focused_window = Some(owner);
            self.focused_monitor = idx;
            self.focused_maximized = plan.is_maximized(owner);
            self.update_border(plan);
        } else {
            // Not managed and not owned by anything managed. It may be
            // an application whose creation we never heard about; if it
            // is not tileable (the task switcher, the shell, system
            // dialogs) the border stays where it was.
            self.try_adopt(plan, window);
        }
    }
}
