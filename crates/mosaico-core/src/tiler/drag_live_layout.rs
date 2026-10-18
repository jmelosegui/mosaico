//! What dropping a dragged window would leave of the tiles of a monitor
//! now, which `drag_live.rs` lays out live.

use crate::{Rect, WindowId};

use super::super::super::bar_states::fills;
use super::super::super::{Plan, Tiler};
use super::super::Drag;
use super::super::target::Target;

impl Tiler {
    /// The monitor whose tiles dropping the window `drag` is about would
    /// move now, with the pointer where the desktop has it, and where each
    /// window of its workspace on screen would then be, the one dragged
    /// included: its own monitor, with the splits its held edges would move
    /// (`move_splits`), or another monitor, with the window put where it
    /// would join (`drop_on`). `None` when the drop would move no tile, as
    /// a resize that moves no split, a swap or a window back in its slot. A
    /// window gone or minimized without a report leaves that workspace
    /// first, as at the drop.
    pub(super) fn live_layout(
        &mut self,
        plan: &mut Plan,
        drag: &Drag,
    ) -> Option<(usize, Vec<(WindowId, Rect)>)> {
        let pointer = plan.cursor();
        if let Some(splits) = self.splits_dropped(plan, drag, pointer) {
            let ws = self.monitors.get(drag.monitor)?.active_ws();
            let positions = self.positions_of(drag.monitor, ws.windows(), &splits);
            return Some((drag.monitor, positions));
        }
        match self.drop_target(plan, drag, pointer)? {
            Target::Join { monitor, .. } if self.has_filling(plan, monitor) => None,
            Target::Join { monitor, .. } => {
                // Decided again on the layout the preview and the drop see.
                self.prune_stale_handles(plan, monitor);
                let Target::Join { monitor, before } = self.drop_target(plan, drag, pointer)?
                else {
                    return None;
                };
                let positions = self.positions_joining(monitor, before, drag.window)?;
                Some((monitor, positions))
            }
            Target::Back | Target::Swap(_) => None,
        }
    }

    /// Whether a window of the workspace on screen of monitor `idx` is
    /// maximized or fills the monitor, as a fullscreen one does, so that
    /// only a drop there may change it, as it would stop being so.
    fn has_filling(&self, plan: &Plan, idx: usize) -> bool {
        let Some(mon) = self.monitors.get(idx) else {
            return false;
        };
        (mon.active_ws().windows().iter())
            .any(|&window| fills(mon, plan.is_maximized(window), plan.rect(window)))
    }
}

#[cfg(test)]
#[path = "tests/drag_live_drop_tests.rs"]
mod drop_tests;

#[cfg(test)]
#[path = "tests/drag_live_drop_end_tests.rs"]
mod drop_end_tests;

#[cfg(test)]
#[path = "tests/drag_live_drop_cases_tests.rs"]
mod drop_cases_tests;

#[cfg(test)]
#[path = "tests/drag_live_drop_stack_tests.rs"]
mod drop_stack_tests;

#[cfg(test)]
#[path = "tests/drag_live_maximized_tests.rs"]
mod maximized_tests;

#[cfg(test)]
#[path = "tests/drag_live_gap_tests.rs"]
mod gap_tests;

#[cfg(test)]
#[path = "tests/drag_live_gone_tests.rs"]
mod gone_tests;

#[cfg(test)]
#[path = "tests/drag_live_swap_tests.rs"]
mod swap_tests;
