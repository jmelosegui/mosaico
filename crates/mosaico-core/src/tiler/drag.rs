//! What the mouse does in the tiler, and the window the user drags.
//!
//! A drag is about the window the platform reports being dragged, never
//! whichever window has focus, and what it is, a move or a resize, is
//! known from what the pointer grabbed as it started. The tiler follows
//! the drag of a tiled window from its start to its end. Meanwhile the
//! user has the window: no layout pass places it, so it never animates
//! either, and once it has left where it started its border is hidden
//! (`drag_border.rs`). What was held back for the window is dropped as
//! the drag starts, and the pointer is left alone until it ends, whatever
//! moves focus meanwhile (`applier_drag.rs`).
//!
//! The drag ends with its `Moved`, and the window lands where it is
//! dropped (`drag_drop.rs`), or before it, as `drag_end.rs` says. A
//! preview shows where it would land meanwhile (`drag_preview.rs`), and
//! the tiles its drop would move follow it (`drag_live.rs`). A window
//! resized by edges that border splits moves them (`drag_resize.rs`).

use crate::event::{DragStart, Grab};
use crate::{Rect, WindowId};

use super::{Effect, Plan, Tiler};

#[path = "drag_border.rs"]
mod border;
#[path = "drag_drop.rs"]
mod dropping;
#[path = "drag_end.rs"]
mod end;
#[path = "drag_live.rs"]
mod live;
#[path = "drag_mouse.rs"]
mod mouse;
#[path = "drag_preview.rs"]
mod preview;
#[path = "drag_resize.rs"]
mod resize;
#[path = "drag_slots.rs"]
mod slots;
#[path = "drag_target.rs"]
mod target;

pub(super) use mouse::Mouse;

/// The drag of a tiled window, as the tiler follows it.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Drag {
    /// The window the user drags.
    pub(super) window: WindowId,
    /// The monitor it is tiled on.
    pub(super) monitor: usize,
    /// What the pointer holds it by: its title bar for a window that was
    /// maximized as the drag started, which can only be moved.
    pub(super) grab: Option<Grab>,
    /// Whether the window was maximized as the drag started.
    pub(super) from_maximized: bool,
    /// Where the window was as the drag started.
    pub(super) start: Rect,
    /// Where the pointer was as the drag started, if the platform read it.
    pub(super) pointer: Option<(i32, i32)>,
    /// Whether a report found the window away from where it started: its
    /// border is hidden from then on.
    pub(super) left_start: bool,
    /// Whether the tiler decides where the window lands, and whether a
    /// resize moves the splits its edges border, until an action, from a
    /// key, a command or a bar, or a display change interrupts the drag.
    pub(super) decides: bool,
    /// Where the preview shows the window would land, while it shows.
    pub(super) preview: Option<Rect>,
}

impl Tiler {
    /// Starts following the drag of `window`, which began as `start` says,
    /// when the window is tiled on the workspace on screen. Any drag
    /// followed before ends: the user drags one window at a time. Whether
    /// the window was maximized is what the platform saw as the drag
    /// started, when it could tell, as the drag may have taken it out of
    /// maximized by now.
    pub(super) fn start_drag(&mut self, plan: &mut Plan, window: WindowId, start: &DragStart) {
        self.end_drag(plan);
        let Some((monitor, ws)) = self.find_window(window) else {
            return;
        };
        if ws != self.monitors[monitor].active_workspace {
            return;
        }
        let Some(start_rect) = start.rect.or_else(|| plan.rect(window)) else {
            return;
        };
        let from_maximized = start.maximized.unwrap_or_else(|| plan.is_maximized(window));
        let grab = if from_maximized {
            Some(Grab::TitleBar)
        } else {
            start.grab
        };
        crate::log_debug!("drag {} from mon {} by {:?}", window, monitor, grab);
        self.mouse.drag = Some(Drag {
            window,
            monitor,
            grab,
            from_maximized,
            start: start_rect,
            pointer: start.pointer,
            left_start: false,
            decides: true,
            preview: None,
        });
    }

    /// Stops following the drag, if the tiler follows one, hides its
    /// preview, and returns it.
    pub(super) fn end_drag(&mut self, plan: &mut Plan) -> Option<Drag> {
        let drag = self.mouse.drag.take()?;
        crate::log_debug!("drag {} ends", drag.window);
        if drag.preview.is_some() {
            plan.emit(Effect::HidePreview);
        }
        Some(drag)
    }

    /// Whether the tiler follows the drag of `window`: until it ends,
    /// nothing places the window.
    pub(super) fn is_dragged(&self, window: WindowId) -> bool {
        self.mouse.drag.as_ref().is_some_and(|d| d.window == window)
    }
}

#[cfg(test)]
#[path = "tests/drag_support.rs"]
mod support;

#[cfg(test)]
#[path = "tests/drag_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/drag_end_tests.rs"]
mod end_tests;

#[cfg(test)]
#[path = "tests/drag_hold_tests.rs"]
mod hold_tests;

#[cfg(test)]
#[path = "tests/drag_start_tests.rs"]
mod start_tests;

#[cfg(test)]
#[path = "tests/drag_bar_tests.rs"]
mod bar_tests;
