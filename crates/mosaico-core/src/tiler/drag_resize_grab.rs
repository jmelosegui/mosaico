//! The edges a resize held when the platform could not tell its grab, as
//! for a window that runs its own move and resize loops, which no hit test
//! at the pointer describes: they are read from how the drag changed the
//! window.

use crate::Rect;
use crate::event::{Edges, Grab};
use crate::motion::MIN_TRAVEL;

use super::super::super::{Plan, Tiler};
use super::super::Drag;

impl Tiler {
    /// The edges the user held the window `drag` is about by: those the
    /// platform told, or, when it could not tell, those [`inferred_edges`]
    /// reads from where the window started and where it is, for a window
    /// that still lies on its own monitor. None for a drag by the title
    /// bar, and for one that changed the window otherwise than a resize.
    pub(in crate::tiler) fn held_edges(&self, plan: &Plan, drag: &Drag) -> Option<Edges> {
        match drag.grab {
            Some(Grab::Edges(edges)) => Some(edges),
            Some(Grab::TitleBar) => None,
            None => {
                let rect = plan.rect(drag.window)?;
                let edges = inferred_edges(drag.start, rect)
                    .filter(|_| self.lies_on(drag.monitor, rect))?;
                crate::log_debug!("drag {} read as a resize by {:?}", drag.window, edges);
                Some(edges)
            }
        }
    }
}

/// The edges held by a drag that changed a window from `start` to `rect`:
/// on each axis, the edge that moved more than [`MIN_TRAVEL`] pixels while
/// the other edge of that axis stayed within them. None when no edge moved
/// that far, and when both edges of an axis did, as a move, a rescale on
/// the way to a monitor of another scale or a snap moves them, which is no
/// resize.
pub(in crate::tiler) fn inferred_edges(start: Rect, rect: Rect) -> Option<Edges> {
    let moved =
        |from: i32, to: i32| (i64::from(to) - i64::from(from)).abs() > i64::from(MIN_TRAVEL);
    let axis = |near: bool, far: bool| (!(near && far)).then_some((near, far));
    let (left, right) = axis(
        moved(start.x, rect.x),
        moved(start.x + start.width, rect.x + rect.width),
    )?;
    let (top, bottom) = axis(
        moved(start.y, rect.y),
        moved(start.y + start.height, rect.y + rect.height),
    )?;
    let edges = Edges {
        left,
        top,
        right,
        bottom,
    };
    (left || top || right || bottom).then_some(edges)
}

#[cfg(test)]
#[path = "tests/drag_resize_grab_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/drag_resize_unknown_cases_tests.rs"]
mod cases_tests;
