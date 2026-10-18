//! Where a window dragged over another monitor goes while the pointer is
//! in a gap between the tiles there.
//!
//! Each report notes where the window would join another monitor, if it
//! would, and the pointer in a gap between two of its tiles, inside the
//! box they fill, keeps that target: before the tile it was over, or at
//! the end. The tiles that made room stay put, the preview with them, and
//! a drop there lands as they show, so nothing bounces as the pointer
//! crosses from one tile to the next. Over an outer gap or the bar's
//! strip the window joins the end, as it does in a gap with no target
//! noted, as before a first report.
//!
//! Over a tile, the window joins where the slot it gets holds the pointer,
//! so the room the tiles make opens under it: where it was going while
//! that slot still does, else before that tile, after it, or anywhere that
//! does, else before that tile.

use crate::WindowId;

use super::super::{Plan, Tiler};
use super::Drag;
use super::target::{DropArea, Target, contains, target};

impl Tiler {
    /// Notes where the window the user drags would join another monitor if
    /// dropped now, with the pointer where the desktop has it, for the
    /// tiles, the preview and the drop to keep in a gap between tiles.
    pub(in crate::tiler) fn aim_drag(&mut self, plan: &Plan) {
        let Some(drag) = self.mouse.drag.as_ref().filter(|d| d.decides) else {
            return;
        };
        let joining = self
            .drop_target(plan, drag, plan.cursor())
            .filter(|t| matches!(t, Target::Join { .. }));
        if let Some(drag) = self.mouse.drag.as_mut() {
            drag.joining = joining;
        }
    }

    /// Where the window `drag` is about lands dropped with the pointer at
    /// `pointer`, as [`target`] has it, joining another monitor where its
    /// slot holds the pointer ([`join_under`](Self::join_under)).
    pub(in crate::tiler) fn lands_at(&self, drag: &Drag, pointer: (i32, i32)) -> Option<Target> {
        match target(drag, pointer, &self.drop_areas(drag.monitor))? {
            Target::Join { monitor, before } => Some(Target::Join {
                monitor,
                before: self.join_under(drag, monitor, before, pointer),
            }),
            other => Some(other),
        }
    }

    /// Where the window `drag` is about joins monitor `idx`, `before` a
    /// tile there or at the end, so that the slot it gets holds `pointer`:
    /// where the last report had it while that slot still does, so the
    /// tiles that made room stay put, else before the tile `before` names,
    /// after it, or anywhere else that does, in that order, else `before`.
    pub(in crate::tiler) fn join_under(
        &self,
        drag: &Drag,
        idx: usize,
        before: Option<WindowId>,
        pointer: (i32, i32),
    ) -> Option<WindowId> {
        let holds = |at: Option<WindowId>| {
            let slot = self.slot_joining(idx, at, drag.window);
            slot.is_some_and(|rect| contains(rect, pointer))
        };
        let noted = match drag.joining {
            Some(Target::Join { monitor, before }) if monitor == idx => Some(before),
            _ => None,
        };
        let windows = self.monitors.get(idx).map(|m| m.active_ws().windows());
        let windows = windows.unwrap_or_default();
        let next = |w: WindowId| {
            let at = windows.iter().position(|&o| o == w)?;
            Some(windows.get(at + 1).copied())
        };
        let after = before.and_then(next);
        let anywhere = windows.iter().map(|&w| Some(w)).chain([None]);
        let mut tried = noted
            .into_iter()
            .chain([before])
            .chain(after)
            .chain(anywhere);
        tried.find(|&at| holds(at)).unwrap_or(before)
    }
}

/// The tile the window `drag` is about was going before on `monitor`,
/// `on`, when `pointer` is in a gap between its tiles, inside the box they
/// fill: what the last report noted, if it joined that monitor.
pub(super) fn kept(
    drag: &Drag,
    monitor: usize,
    on: &DropArea,
    (x, y): (i32, i32),
) -> Option<WindowId> {
    let rects = || on.tiles.iter().flatten().map(|&(_, r)| r);
    let left = rects().map(|r| r.x).min()?;
    let top = rects().map(|r| r.y).min()?;
    let right = rects().map(|r| r.x + r.width).max()?;
    let bottom = rects().map(|r| r.y + r.height).max()?;
    let inside = x >= left && x < right && y >= top && y < bottom;
    match drag.joining {
        Some(Target::Join { monitor: m, before }) if inside && m == monitor => before,
        _ => None,
    }
}
