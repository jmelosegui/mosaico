//! Where a window the user drops lands, and how the drag ended: plain
//! decisions on where the pointer is, what the tiler has laid out, and
//! how the drag left the window.

use crate::event::Grab;
use crate::motion::hardly_apart;
use crate::{Rect, WindowId};

use super::Drag;

/// What a drop target is worked out from, for one monitor.
#[derive(Debug, Clone, PartialEq)]
pub(in crate::tiler) struct DropArea {
    /// Where the pointer counts as on the monitor: its work area, the
    /// bar's strip included.
    pub(in crate::tiler) area: Rect,
    /// Where each window of the workspace on screen is laid out, in
    /// order, or `None` in monocle, where a window dropped joins the end.
    pub(in crate::tiler) tiles: Option<Vec<(WindowId, Rect)>>,
}

/// Where a window the user drops lands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::tiler) enum Target {
    /// Back in its slot.
    Back,
    /// In the slot of another window of its monitor, which takes its slot.
    Swap(WindowId),
    /// On the workspace on screen of another monitor: before a window
    /// there, or at the end.
    Join {
        /// The monitor.
        monitor: usize,
        /// The window it goes before, if any.
        before: Option<WindowId>,
    },
}

/// Where the window `drag` is about lands when dropped at `pointer`: on
/// its own monitor, in the slot of the tile under the pointer, or back in
/// its own when there is none, as over a gap or in monocle; on another
/// monitor, before the tile under the pointer, or at the end. `None` when
/// the pointer is on no monitor.
pub(in crate::tiler) fn target(
    drag: &Drag,
    pointer: (i32, i32),
    monitors: &[DropArea],
) -> Option<Target> {
    let (index, on) = monitors
        .iter()
        .enumerate()
        .find(|(_, m)| contains(m.area, pointer))?;
    let under = on
        .tiles
        .iter()
        .flatten()
        .find(|&&(_, rect)| contains(rect, pointer))
        .map(|&(window, _)| window);
    Some(if index != drag.monitor {
        Target::Join {
            monitor: index,
            before: under,
        }
    } else {
        match under {
            Some(other) if other != drag.window => Target::Swap(other),
            _ => Target::Back,
        }
    })
}

/// Whether the end of `drag` leaves everything as it was, to the few
/// pixels a press may wobble ([`hardly_apart`]): the pointer where the drag
/// started it, as a click on the title bar does, or the window, at `rect`,
/// where it started or in its slot, `slot`, where Esc or the system puts it
/// back, the slot as the start may have been read once the window moved.
pub(in crate::tiler) fn cancelled(
    drag: &Drag,
    rect: Rect,
    pointer: Option<(i32, i32)>,
    slot: Option<Rect>,
) -> bool {
    let at = |(x, y): (i32, i32)| Rect::new(x, y, 0, 0);
    let near = |(p, s): ((i32, i32), (i32, i32))| hardly_apart(at(p), at(s));
    let clicked = pointer.zip(drag.pointer).is_some_and(near);
    clicked || hardly_apart(rect, drag.start) || Some(rect) == slot
}

/// How a drag that was not cancelled ended, from how it started and where
/// it left the window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::tiler) enum Ending {
    /// The drag resized the window.
    Resized,
    /// The system snapped or maximized the window on its own monitor as it
    /// was dropped.
    Snapped,
    /// The drag moved the window.
    Moved,
}

/// How `drag` ended, leaving its window at `rect`, maximized or not, with
/// the pointer and the window on its own monitor when `at_home`.
///
/// A drag that grabbed an edge resizes. One whose grab the platform could
/// not tell resized when the size changed. A move keeps the window's size,
/// to a few pixels on every edge, but for the system rescaling it on a
/// monitor of another scale, so a move that changed it more on its own
/// monitor, or left it maximized there, was snapped by the system. One the
/// system maximized on another monitor has moved there. A snap that leaves
/// the window its size, as to the half of the monitor the other of two
/// tiles fills without gaps, is taken for a move onto that tile, which
/// puts the window where the system did. A window that was maximized as
/// the drag started changes size as it leaves maximized, and has moved all
/// the same.
pub(in crate::tiler) fn ending(drag: &Drag, rect: Rect, maximized: bool, at_home: bool) -> Ending {
    let resized = (rect.width, rect.height) != (drag.start.width, drag.start.height);
    match drag.grab {
        Some(Grab::Edges(_)) => Ending::Resized,
        _ if maximized && at_home => Ending::Snapped,
        _ if maximized || !resized || drag.from_maximized => Ending::Moved,
        None => Ending::Resized,
        Some(_) if at_home && !hardly_resized(drag.start, rect) => Ending::Snapped,
        Some(_) => Ending::Moved,
    }
}

/// Whether `rect` is the size of `start`, to no more than a few pixels on
/// any edge, as [`hardly_apart`] counts them.
fn hardly_resized(start: Rect, rect: Rect) -> bool {
    let (x, y) = (start.x, start.y);
    hardly_apart(Rect { x, y, ..rect }, start)
}

/// Whether `rect` holds the point `(x, y)`.
pub(in crate::tiler) fn contains(rect: Rect, (x, y): (i32, i32)) -> bool {
    x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
}

#[cfg(test)]
#[path = "tests/drag_target_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/drag_ending_tests.rs"]
mod ending_tests;

#[cfg(test)]
#[path = "tests/drag_cancel_tests.rs"]
mod cancel_tests;
