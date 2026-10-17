//! The pointer while the user drags a window.
//!
//! The user moves the pointer, and the window with it, until the drag
//! ends, so nothing else may move it meanwhile: a pointer that jumped
//! would take the window along. This holds for any window they drag,
//! tiled or not. A focus change is made as it comes, but the pointer
//! stays where the user has it, and goes where the last move asked for
//! once the drag ends, after what its end does to the layout, unless
//! focus has moved on from there without the pointer meanwhile, as a
//! click or the wheel on a bar moves it. A drag whose window closes,
//! hides or is minimized holds the pointer no more, and the move it was
//! asked for is dropped: what the window leaving does moves the pointer
//! if it has to. So it is once another window takes focus other than by
//! the tiler's doing, which ends a drag whose end the platform never
//! reported: the same reports as end a drag the tiler follows
//! (`drag_end.rs`), so the two always agree. Focus follows mouse rests
//! meanwhile, as moving focus while the user holds a window would reach
//! what moves or sizes it: the pointer resting on a window is not
//! reported to the tiler then (`applier_reports.rs`).

use std::time::Instant;

use crate::{WindowEvent, WindowId};

use super::super::{Desktop, Effect, Platform};
use super::Applier;

/// The drag that holds the pointer, and where the pointer goes once it
/// ends.
#[derive(Debug, Default)]
pub(super) struct PointerHold {
    /// The window the user drags, while they do.
    dragged: Option<WindowId>,
    /// The window the pointer goes to once the drag ends.
    owed: Option<WindowId>,
}

impl Applier {
    /// Takes what `event` says of a drag: the user grabbing a window holds
    /// the pointer, a new drag in place of one whose end was never
    /// reported, and the window leaving lets it go, when `desktop`
    /// agrees that it left. So does a report that another window took
    /// focus, unless it is an `echo`, the window the tiler has `focused`,
    /// or a window that one owns, as a dialog the system gives focus in
    /// its place.
    pub(super) fn drag_seen(
        &mut self,
        event: &WindowEvent,
        echo: bool,
        focused: Option<WindowId>,
        desktop: &dyn Desktop,
    ) {
        let hold = &mut self.pointer;
        match *event {
            WindowEvent::MoveStarted { window, .. } => hold.dragged = Some(window),
            WindowEvent::Destroyed { window } => hold.left(window),
            WindowEvent::Hidden { window } if !desktop.is_visible(window) => hold.left(window),
            WindowEvent::Minimized { window } if desktop.is_minimized(window) => hold.left(window),
            WindowEvent::Focused { window } if !echo => {
                hold.focused(window, desktop.owner(window), focused);
            }
            _ => {}
        }
    }

    /// Whether the user drags a window, so that nothing may move the
    /// pointer.
    pub(super) fn holds_pointer(&self) -> bool {
        self.pointer.dragged.is_some()
    }

    /// Has the pointer go to `window` once the drag ends, instead of an
    /// earlier window it was to go to.
    pub(super) fn owe_pointer(&mut self, window: WindowId) {
        self.pointer.owed = Some(window);
    }

    /// Ends the drag of `window`, if it holds the pointer, and puts the
    /// pointer where it was last asked to go meanwhile: at the center of
    /// that window, once a placement held back for it is made, if it is
    /// still the `focused` one. Focus moved on without the pointer, as a
    /// click on a bar moves it, leaves the pointer where the user has it.
    pub(in crate::tiler) fn drag_ended(
        &mut self,
        window: WindowId,
        focused: Option<WindowId>,
        platform: &mut dyn Platform,
        now: Instant,
    ) {
        if self.pointer.dragged != Some(window) {
            return;
        }
        self.pointer.dragged = None;
        if let Some(owed) = self.pointer.owed.take().filter(|&w| Some(w) == focused)
            && let Some(effect) = self.held.admit(Effect::CenterCursor { window: owed }, now)
        {
            self.run(&effect, platform, now);
        }
    }
}

impl PointerHold {
    /// Lets the pointer go, without moving it, when `window` is the one
    /// dragged and has left.
    fn left(&mut self, window: WindowId) {
        if self.dragged == Some(window) {
            *self = Self::default();
        }
    }

    /// Lets the pointer go, without moving it, when `window`, owned by
    /// `owner`, took focus, unless it is the one dragged, the one the
    /// tiler has `focused`, or a window that one owns.
    fn focused(&mut self, window: WindowId, owner: Option<WindowId>, focused: Option<WindowId>) {
        let ours = focused.is_some_and(|f| f == window || Some(f) == owner);
        if self.dragged.is_some_and(|w| w != window) && !ours {
            *self = Self::default();
        }
    }
}

#[cfg(test)]
#[path = "tests/applier_drag_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/applier_drag_end_tests.rs"]
mod end_tests;

#[cfg(test)]
#[path = "tests/applier_drag_bar_tests.rs"]
mod bar_tests;

#[cfg(test)]
#[path = "tests/applier_drag_focus_tests.rs"]
mod focus_tests;
