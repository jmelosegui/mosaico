//! What dropping a dragged window would leave of the tiles of a monitor
//! now, which `drag_live.rs` lays out live.

use crate::{Rect, WindowId};

use super::super::super::{Plan, Tiler};
use super::super::Drag;

impl Tiler {
    /// The monitor whose tiles dropping the window `drag` is about would
    /// move now, with the pointer where the desktop has it, and where each
    /// window of its workspace on screen would then be, the one dragged
    /// included: its own monitor, with the splits its held edges would move
    /// (`move_splits`). `None` when the drop would move no tile, as a
    /// resize that moves no split or a move. A window gone or minimized
    /// without a report leaves that workspace first, as at the drop.
    pub(super) fn live_layout(
        &mut self,
        plan: &mut Plan,
        drag: &Drag,
    ) -> Option<(usize, Vec<(WindowId, Rect)>)> {
        let pointer = plan.cursor();
        let splits = self.splits_dropped(plan, drag, pointer)?;
        let ws = self.monitors.get(drag.monitor)?.active_ws();
        let positions = self.positions_of(drag.monitor, ws.windows(), &splits);
        Some((drag.monitor, positions))
    }
}
