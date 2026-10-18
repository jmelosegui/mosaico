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

use crate::WindowId;

use super::super::{Plan, Tiler};
use super::Drag;
use super::target::{DropArea, Target};

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
