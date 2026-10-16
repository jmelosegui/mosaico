//! The frames of a transition, and what became of them.

use crate::{Rect, WindowId};

/// Where a window should be at a moment of its transition, for
/// [`FrameControl::place_frames`](super::FrameControl::place_frames).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Frame {
    /// The window.
    pub window: WindowId,
    /// Its visible bounds at this moment.
    pub rect: Rect,
    /// Whether this is the first frame of the window's motion. The
    /// platform then reads the window again, such as its invisible
    /// borders. A window whose owner changed, a new window given the id of
    /// an old one, starts over with its first frame, and is reported gone
    /// for any other.
    pub first: bool,
}

/// What [`FrameControl::place_frames`](super::FrameControl::place_frames)
/// found when a frame came for a window, and so whether it posted it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameOutcome {
    /// The frame was posted, or was not needed, as one the window can
    /// only take at the rect it is at, or the system refused the post,
    /// which then holds nothing back.
    Handled {
        /// The window's visible bounds before this frame lands: where
        /// the frames before took it, or wherever its application or the
        /// system put it instead.
        reached: Rect,
    },
    /// Nothing was posted: the frame before was posted less than
    /// [`PENDING_LIMIT`](super::PENDING_LIMIT) ago and the window has not
    /// been seen to move since, or its application is not responding.
    Pending,
    /// The window is maximized, and was left alone, as the caller leaves
    /// it: a frame queued before it was maximized may still land.
    Maximized,
    /// The window is minimized, and was left alone, as the caller leaves
    /// it: a frame queued before it was minimized may still land.
    Minimized,
    /// The window no longer exists, or is another window now.
    Gone,
}
