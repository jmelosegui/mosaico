//! What the end of a drag does.
//!
//! With `[mouse] drag_to_swap`, a window the tiler followed the drag of
//! lands where the pointer drops it (`drag_target.rs`): in the slot of
//! another tile of its monitor, which takes its slot, or on another
//! monitor's workspace, before the tile under the pointer or at the end.
//! Dropped over its own slot, a gap or a monocle workspace of its own
//! monitor, or snapped or maximized there by the system, it goes back to
//! its slot. One the system maximized on another monitor joins it, and
//! stays maximized there, and one it rescaled across two monitors has
//! moved, its size changed or not. A drag that was cancelled or was a
//! click, one that resized the window, one the tiler did not follow or
//! that an action, from a key, a command or a bar, or a display change
//! interrupted, and one whose pointer could not be read are handled as
//! every drag was before: the window goes back to its slot, or joins the
//! end of the workspace of the monitor it is on now.

use crate::WindowId;

use super::super::{Plan, Tiler, Timing};
use super::Drag;
use super::target::{Ending, Target, cancelled, ending, target};

impl Tiler {
    /// Handles the end of a drag of `window`, with the pointer at `pointer`
    /// as the platform read it then.
    pub(in crate::tiler) fn on_moved(
        &mut self,
        plan: &mut Plan,
        window: WindowId,
        pointer: Option<(i32, i32)>,
    ) {
        let drag = self
            .end_drag(plan)
            .filter(|d| d.window == window && d.decides);
        let dropped =
            drag.and_then(|drag| self.drop_target(plan, &drag, pointer).map(|t| (drag, t)));
        if let Some((drag, target)) = dropped {
            self.drop_on(plan, window, drag.monitor, target);
            if self.focused_window == Some(window) {
                self.focused_maximized = plan.is_maximized(window);
            }
            return;
        }
        // A window the user just maximized stays maximized; retiling it
        // would snap it back into the layout.
        let maximized = plan.is_maximized(window);
        if self.focused_window == Some(window) {
            self.focused_maximized = maximized;
        }
        if maximized {
            self.update_border(plan);
        } else {
            self.reassign_monitor(plan, window);
        }
    }

    /// Where the window `drag` is about lands when dropped with the pointer
    /// at `pointer`, or `None` when its end is handled as before drops
    /// moved windows. The preview follows the same rule.
    pub(super) fn drop_target(
        &self,
        plan: &Plan,
        drag: &Drag,
        pointer: Option<(i32, i32)>,
    ) -> Option<Target> {
        if !self.mouse.drag_to_swap {
            return None;
        }
        let rect = plan.rect(drag.window)?;
        let slot = self.slot_of(drag.monitor, drag.window);
        if cancelled(drag, rect, pointer, slot) {
            return None;
        }
        let lands = pointer.and_then(|at| target(drag, at, &self.drop_areas()));
        // A window across two monitors was rescaled on its way, not snapped.
        let home = self.lies_on(drag.monitor, rect);
        let at_home = home && matches!(lands, Some(Target::Back | Target::Swap(_)));
        match ending(drag, rect, plan.is_maximized(drag.window), at_home) {
            Ending::Resized => None,
            Ending::Snapped => Some(Target::Back),
            Ending::Moved => lands,
        }
    }

    /// Puts `window`, dropped as the drag from monitor `from` ends, where
    /// `target` says, and lays out the monitors it leaves and joins. It
    /// keeps focus, and the pointer stays where the user dropped it.
    fn drop_on(&mut self, plan: &mut Plan, window: WindowId, from: usize, target: Target) {
        crate::log_info!("drop {} from mon {}: {:?}", window, from, target);
        match target {
            Target::Back => self.apply_layout_on(plan, from),
            Target::Swap(other) => {
                let ws = self.monitors[from].active_ws_mut();
                if let (Some(a), Some(b)) = (ws.index_of(window), ws.index_of(other)) {
                    ws.swap(a, b);
                }
                self.apply_layout_on(plan, from);
            }
            Target::Join { monitor, before } => {
                self.monitors[from].active_ws_mut().remove(window);
                let ws = self.monitors[monitor].active_ws_mut();
                let at = before.and_then(|w| ws.index_of(w)).unwrap_or(ws.len());
                ws.insert(at, window);
                if self.focused_window == Some(window) {
                    self.focused_monitor = monitor;
                }
                // Both monitors are laid out before any border is drawn, so
                // the window's border never shows at its old slot. One the
                // system maximized stays so, as one the user maximizes does.
                self.apply_layout_positions(plan, from, Timing::Now);
                if !plan.is_maximized(window) {
                    self.apply_layout_positions(plan, monitor, Timing::Now);
                }
                self.update_border(plan);
            }
        }
    }
}

#[cfg(test)]
#[path = "tests/drag_drop_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/drag_drop_cases_tests.rs"]
mod cases_tests;

#[cfg(test)]
#[path = "tests/drag_drop_monitor_tests.rs"]
mod monitor_tests;

#[cfg(test)]
#[path = "tests/drag_drop_off_tests.rs"]
mod off_tests;

#[cfg(test)]
#[path = "tests/drag_drop_motion_tests.rs"]
mod motion_tests;
