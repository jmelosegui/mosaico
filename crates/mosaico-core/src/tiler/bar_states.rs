//! What each monitor's bar shows of the tiler's state, and whether a
//! window under it takes the pointer.

use crate::bar::BarState;
use crate::{Rect, WindowId};

use super::{Desktop, MonitorState, Tiler};

#[path = "bar_filling.rs"]
mod filling;

pub(super) use filling::Filling;

impl Tiler {
    /// What each monitor's bar shows, in monitor order.
    ///
    /// `update_text` and `paused` come from the daemon and go to every
    /// monitor. Only the monitor the user is working on shows the focused
    /// window. No monitor is marked as having a maximized window: that
    /// takes the desktop, see [`mark_maximized`](Self::mark_maximized).
    pub fn bar_states(&self, update_text: &str, paused: bool) -> Vec<BarState> {
        self.monitors
            .iter()
            .enumerate()
            .map(|(i, mon)| BarState {
                active_workspace: mon.active_workspace,
                workspace_count: mon.workspaces.len(),
                layout: mon.active_ws().layout_kind(),
                monocle: mon.active_ws().monocle(),
                focused_window: if i == self.focused_monitor {
                    self.focused_window
                } else {
                    None
                },
                update_text: update_text.to_string(),
                paused,
                wrap: self.workspace_wrap,
                maximized: false,
            })
            .collect()
    }

    /// Marks each of `states`, in monitor order, with whether a window
    /// under its bar is maximized or fullscreen, as `desktop` has it now:
    /// any window on screen there, managed or not, but not one the tiler
    /// manages on a workspace that is not on screen, however the desktop
    /// reports it.
    pub fn mark_maximized(&self, states: &mut [BarState], desktop: &dyn Desktop) {
        for (_, at) in self.filling_windows(desktop) {
            if let Some(state) = states.get_mut(at) {
                state.maximized = true;
            }
        }
    }

    /// The windows that [`mark_maximized`](Self::mark_maximized) counts,
    /// each with the index of the monitor it fills, as `desktop` has them
    /// now: every one, not only the first a monitor has.
    fn filling_windows(&self, desktop: &dyn Desktop) -> Vec<(WindowId, usize)> {
        let monitor = |window| {
            let id = desktop.monitor_of(window);
            self.monitors.iter().position(|mon| mon.id == id)
        };
        let fills_it = |&(window, at): &(WindowId, usize)| {
            let (maximized, rect) = (desktop.is_maximized(window), desktop.rect(window));
            fills(&self.monitors[at], maximized, rect)
        };
        (desktop.windows_on_screen().into_iter())
            .filter(|&window| self.shows(window))
            .filter_map(|window| Some((window, monitor(window)?)))
            .filter(fills_it)
            .collect()
    }

    /// Whether `window` is on screen as far as the tiler is concerned:
    /// one it does not manage, or one of a workspace on screen.
    fn shows(&self, window: WindowId) -> bool {
        self.find_window(window)
            .is_none_or(|(mon, ws)| self.monitors[mon].active_workspace == ws)
    }
}

/// Whether a window on `mon`, `maximized` or not, at `rect`, wants the
/// pointer under the monitor's bar: maximized, or over the whole of the
/// monitor's work area, as a fullscreen window is.
pub(in crate::tiler) fn fills(mon: &MonitorState, maximized: bool, rect: Option<Rect>) -> bool {
    maximized || rect.is_some_and(|rect| covers(&rect, &mon.os_work_area))
}

/// Whether `outer` holds all of `inner`.
fn covers(outer: &Rect, inner: &Rect) -> bool {
    outer.x <= inner.x
        && outer.y <= inner.y
        && outer.x + outer.width >= inner.x + inner.width
        && outer.y + outer.height >= inner.y + inner.height
}

#[cfg(test)]
#[path = "tests/bar_state_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/bar_maximized_tests.rs"]
mod maximized_tests;

#[cfg(test)]
#[path = "tests/bar_maximized_windows_tests.rs"]
mod maximized_windows_tests;

#[cfg(test)]
#[path = "tests/bar_filling_tests.rs"]
mod filling_tests;

#[cfg(test)]
#[path = "tests/bar_seed_tests.rs"]
mod seed_tests;
