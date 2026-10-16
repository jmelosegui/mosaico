//! What a platform does with the frames of transitions.

#[cfg(doc)]
use super::{ASYNC_GRACE, FrameTable, PENDING_LIMIT};
use super::{Frame, FrameOutcome};
use crate::{Result, WindowId};
#[cfg(doc)]
use crate::{WindowControl, WindowEvent};

/// Moves windows through the frames of their transitions.
///
/// Every [`WindowControl`] is one too, so the tiler moves windows frame by
/// frame through the same object it places them with. A platform that
/// cannot do this keeps the defaults: [`place_frames`](Self::place_frames)
/// says so, [`posts_behind`](Self::posts_behind) that nothing is posted,
/// and [`settle_responding`](Self::settle_responding) does nothing.
///
/// # What callers do
///
/// - Nothing waits for a frame to land. A window's other changes keep
///   their order with the frames posted before them, so a motion ends with
///   [`place`](WindowControl::place) and [`repaint`](WindowControl::repaint)
///   at its target, which land after its frames, whatever became of them.
/// - Do not read a change back as done while it may be on its way: while
///   [`posts_behind`](Self::posts_behind) holds for a window, the desktop
///   may not show a change made to it until its application gets to it,
///   so a window placed may still read where it was, or as if settled
///   while its last placement is still queued, and one shown may still
///   read hidden. A posted repaint only repairs what was wrong before the
///   changes queued ahead of it: repaint the window again once it reports
///   where a posted placement took it.
/// - To keep a record of how a change was made, ask
///   [`posts_behind`](Self::posts_behind) right after it: a posted change
///   renews the grace, so `true` then means it was posted, or that the
///   application stopped responding since, which is handled the same way,
///   and `false` that it was made at once.
/// - Focus is never posted. Given right after a posted show, it may reach
///   the window before the window is shown.
/// - Stop sending a window frames before hiding or minimizing it: a frame
///   posted after lands after the change.
/// - Leave a window reported [`Maximized`](FrameOutcome::Maximized) or
///   [`Minimized`](FrameOutcome::Minimized) alone. A frame queued when
///   the user or the system maximized or minimized it may land after
///   that, and the next call reports it so.
/// - Once transitions are off, call [`settle_responding`](Self::settle_responding)
///   now and then, so that windows go back to changes made at once.
///
/// # Contract for platform implementations
///
/// - While [`FrameTable::posts_behind`] holds for a window, a placement,
///   a repaint, [`put_behind`](WindowControl::put_behind), and a change of
///   its show state by [`minimize`](WindowControl::minimize),
///   [`hide`](WindowControl::hide), [`show`](WindowControl::show) or
///   [`reveal`](WindowControl::reveal) is posted behind what was posted to
///   it rather than made at once: a change applied at once, a move or a
///   resize alike, would be overtaken by a frame still queued, which lands
///   after it. Note every change posted but a repaint with
///   [`FrameTable::posted`], and every change of show state with
///   [`FrameTable::posted_show_state`] or [`FrameTable::show_state_made`].
///   Hiding a window the platform's way that does not go through the
///   window's own queue is never posted, and neither is focus.
/// - A posted change does not wait for the application, but in one case.
///   A posted placement ends a maximized state as one made at once does:
///   that of a window maximized when the placement is posted, and that of
///   a window minimized then which a show queued ahead brings back
///   maximized. Where it cannot end it without showing a window that is
///   out of view, or is to be once what was posted lands
///   ([`FrameTable::in_view_once_landed`]), the platform may end it at
///   once instead, which waits for the application, but not while the
///   window is still in view nor for one reported not responding: it then
///   stays maximized, and a later show brings it back as if it had not been
///   placed, a hidden window at the normal bounds it had before.
/// - A posted placement lands with the bounds the window has in its normal
///   state, read with its frames ([`FrameTable::kept_for`]) and matched to
///   the window's scale now, even on a window still minimized or
///   maximized when it was posted.
/// - A frame that lands is reported as the window's position changing
///   ([`WindowEvent::LocationChanged`]), as any move is, and never as the
///   user moving it ([`WindowEvent::Moved`]).
pub trait FrameControl {
    /// Moves and resizes windows to the frames of their transitions,
    /// without waiting for their applications. `outcomes` is cleared,
    /// then gets what was found of each window: one [`FrameOutcome`] per
    /// frame, in order.
    ///
    /// The tiler gives every window in a transition a [`Frame`] about
    /// once per display refresh, with the rect it should have at that
    /// moment, and ends each transition with [`place`](WindowControl::place).
    ///
    /// - Returns at once. A frame posted lands when the application
    ///   handles it, which may be at once, later, or never, and its rect
    ///   is then the window's visible bounds.
    /// - A frame posted less than [`PENDING_LIMIT`] ago to a window not
    ///   seen to move since holds the next one back:
    ///   [`Pending`](FrameOutcome::Pending), and nothing posted. So a slow
    ///   application gets at most one frame that often, never a backlog,
    ///   and any move lets the next one go, as the limit does.
    /// - A window whose application is not responding is pending, and
    ///   gets nothing posted.
    /// - [`Handled`](FrameOutcome::Handled) means the frame was posted, or
    ///   was not needed: once mapped to what the window can take, such as
    ///   its own pixels when it does not follow the display's scale, it
    ///   asks for where the window is. It also covers a post the system
    ///   refused, which holds nothing back.
    /// - The first frame of a motion ([`Frame::first`]) reads the window
    ///   again. A window with another owner starts over with it, and is
    ///   reported gone for any other frame.
    /// - It never raises, focuses, shows or hides a window, and never
    ///   changes whether one is maximized or minimized: such a window is
    ///   left alone, and reported. Nothing is repainted beyond what the
    ///   move itself causes.
    /// - A platform that cannot do this returns
    ///   [`Error::Unsupported`](crate::Error::Unsupported), which is the
    ///   default, and the tiler then places windows without transitions.
    ///   One that can moves nothing when given no frames, so asking with
    ///   none tells which it is.
    fn place_frames(&self, frames: &[Frame], outcomes: &mut Vec<FrameOutcome>) -> Result<()> {
        let _ = (frames, outcomes);
        Err(crate::Error::Unsupported("moving windows frame by frame"))
    }

    /// Whether a change made to `window` now is posted behind what was
    /// posted to it, rather than made at once, as
    /// [`FrameTable::posts_behind`] says: for [`ASYNC_GRACE`] after
    /// anything was last posted to the window, then while the window has
    /// not been seen to move since the last thing posted to it, and, once
    /// it was given a frame, while its application does not respond. Never
    /// for a window never given a frame, or given none since
    /// [`settle_responding`](Self::settle_responding) forgot it; never at
    /// all by default, for a platform that posts nothing.
    ///
    /// A posted change lands after all that was posted before it; when
    /// order is lost is in [`FrameTable::posts_behind`]. Unless something
    /// is posted in between, the answer goes from posted to made at once
    /// as time passes, or once the window is seen to move or its
    /// application responds again, and the other way once its application
    /// stops responding, or a frame is given to it while it does not,
    /// which posts nothing but starts its record.
    fn posts_behind(&self, window: WindowId) -> bool {
        let _ = window;
        false
    }

    /// Forgets every window something was last posted to more than
    /// [`ASYNC_GRACE`] ago, or nothing, whose application responds, so the
    /// changes made to it are made at once until it is given a frame. One
    /// within the grace or not responding stays posted to until a later
    /// call finds it otherwise. Does nothing by default.
    fn settle_responding(&self) {}
}
