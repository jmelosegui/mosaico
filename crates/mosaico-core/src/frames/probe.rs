//! What a platform reads from and does to its windows for their frames.

use crate::{Rect, WindowId};

/// What a platform reads from its windows and does to them, for a
/// [`FrameTable`](super::FrameTable) to post them their frames.
///
/// Where a window is, and where it is moved to, are in whatever terms
/// the platform reads windows in, the same for all of them, such as
/// bounds that include invisible borders. Frames are in visible bounds:
/// [`aim`](Self::aim) and [`visible`](Self::visible) go from one to the
/// other.
pub trait FrameProbe {
    /// What the platform reads about a window with the first frame of its
    /// motion and keeps for the rest of it, such as the invisible borders
    /// around it.
    type Kept: Copy;

    /// Whether the window still exists.
    fn exists(&self, window: WindowId) -> bool;

    /// Whether the window is minimized.
    fn minimized(&self, window: WindowId) -> bool;

    /// Whether the window is maximized.
    fn maximized(&self, window: WindowId) -> bool;

    /// Whether the window is shown, as a change of its show state that
    /// hides it or shows it again leaves it. A window kept out of view
    /// some other way, one that does not change its show state, still is.
    fn shown(&self, window: WindowId) -> bool;

    /// Whether the window's application responds.
    fn responding(&self, window: WindowId) -> bool;

    /// Who owns the window, such as the thread that created it: a window
    /// with another owner than before is a new window, given the id of an
    /// old one. A cheap read, made on every lookup. `None` once it is gone.
    fn owner(&self, window: WindowId) -> Option<u64>;

    /// Where the window is, or `None` once it is gone.
    fn at(&self, window: WindowId) -> Option<Rect>;

    /// What to keep about the window for its motion, or `None` once it
    /// is gone.
    fn keep(&self, window: WindowId) -> Option<Self::Kept>;

    /// Where the frame `rect`, in visible bounds, would put the window:
    /// the rect it can take of it, such as one in its own pixels when it
    /// does not follow the display's scale.
    fn aim(&self, window: WindowId, kept: &Self::Kept, rect: Rect) -> Rect;

    /// The visible bounds of a window at `at`.
    fn visible(&self, kept: &Self::Kept, at: Rect) -> Rect;

    /// Posts the window a move to `to` without waiting for it. False
    /// when the system refused it.
    fn post(&mut self, window: WindowId, to: Rect) -> bool;
}
