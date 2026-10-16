//! The move a window taken in while it is on screen gets ahead of the
//! layout pass (#28).

use crate::{Rect, WindowId};

use super::{Effect, Plan, Tiler};

impl Tiler {
    /// Moves a window just added to the active workspace of monitor
    /// `idx` to its slot ahead of the layout pass, with a move that does
    /// not wait for its application.
    ///
    /// The window is on screen already, wherever the system or the
    /// application put it. The layout pass places the windows in the
    /// layout's order and draws borders as it goes, and each placement
    /// may wait for its application, so a window that comes late in that
    /// order would stay where it is until all of that is done. This move
    /// lands as soon as the window's application handles its messages,
    /// and nothing waits for it. The layout pass then places the window as
    /// it always did, and the applier makes that placement in full, so the
    /// window is repainted even when this move landed first.
    ///
    /// Nothing is emitted in monocle mode, which keeps the order its pass
    /// gives the monocle window and the windows waiting behind it. Nor is
    /// anything emitted when the layout moves the window first anyway,
    /// when the window is in its slot already, as one pre-placed before it
    /// was shown is, or when it is maximized, which only a placement
    /// undoes.
    pub(super) fn move_before_layout(&mut self, plan: &mut Plan, idx: usize, window: WindowId) {
        if plan.is_maximized(window) {
            return;
        }
        let positions = self.positions_on(plan, idx);
        if let Some(&(_, rect)) = positions.iter().skip(1).find(|(w, _)| *w == window)
            && plan.rect(window) != Some(rect)
        {
            plan.emit(Effect::PrePlace { window, rect });
        }
    }

    /// Where the layout pass puts the windows on the monitor's active
    /// workspace, in the order it places them, after pruning the ones
    /// that are gone or minimized. Nothing is emitted.
    ///
    /// Empty in monocle mode, whatever the pass does there: it places the
    /// monocle window alone, after the windows waiting behind it, or the
    /// whole layout when monocle has no window to show.
    fn positions_on(&mut self, plan: &Plan, monitor_idx: usize) -> Vec<(WindowId, Rect)> {
        self.prune_stale_handles(plan, monitor_idx);
        let monocle = self
            .monitors
            .get(monitor_idx)
            .is_some_and(|state| state.active_ws().monocle());
        if monocle {
            return Vec::new();
        }
        self.compute_positions(monitor_idx)
    }
}
