//! The last thing posted to a window, and whether the window has been
//! seen to move since, or seen as a change of its show state leaves it.

use std::time::Instant;

use crate::Rect;

use super::PENDING_LIMIT;

/// Something a [`FrameTable`](super::FrameTable) posted to a window, a
/// frame or another change: when, where the window was then, and whether
/// it has been seen anywhere else since.
///
/// A window that has moved since may have handled it, wherever it ended
/// up: windows round their own size, put it back, or are moved by the
/// system, so the rect posted may never be the rect they reach. It may
/// also still have it queued, moved by something else: the system, the
/// user, or its application. One that has not moved may still have it
/// queued, or may have taken it without moving, and nothing tells the two
/// apart. So nothing depends on knowing whether it landed: a frame only
/// holds the next one back for [`PENDING_LIMIT`], and any move seen, once
/// the grace has run out, ends the order a change keeps with it. A move
/// seen once counts for good, even if the window is back where it was by
/// the next look. Rects are in whatever terms the platform reads windows
/// in, the same for all of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct LastPost {
    /// When it was posted.
    pub(super) when: Instant,
    /// Where the window was then.
    pub(super) from: Rect,
    /// Whether the window has been seen anywhere else since.
    moved: bool,
}

impl LastPost {
    /// Something posted at `when` to the window at `from`.
    pub(super) fn new(when: Instant, from: Rect) -> Self {
        Self {
            when,
            from,
            moved: false,
        }
    }

    /// Notes that the window was seen at `at`.
    pub(super) fn seen_at(&mut self, at: Rect) {
        self.moved |= at != self.from;
    }

    /// Whether the window has not been seen to move since, so that what
    /// was posted may still be queued.
    pub(super) fn unmoved(&self) -> bool {
        !self.moved
    }

    /// Whether, as the last frame posted, it holds the next one back at
    /// `now`: it was posted less than [`PENDING_LIMIT`] before, and the
    /// window has not been seen to move since.
    pub(super) fn holds_back(&self, now: Instant) -> bool {
        self.unmoved() && now.saturating_duration_since(self.when) < PENDING_LIMIT
    }
}

/// The last change of a window's show state posted to it: whether it
/// leaves the window in view, and whether the window has been seen
/// otherwise since it was posted, or was then.
///
/// Seeing the window as the change leaves it only says it landed once the
/// window has been seen otherwise: one posted the other way ahead of it
/// may still be queued, and the window still look as it did before both.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ViewNote {
    /// Whether the change leaves the window in view.
    pub(super) in_view: bool,
    /// Whether the window has been seen otherwise since, or was then.
    seen_otherwise: bool,
}

impl ViewNote {
    /// A change posted that leaves the window in view when `in_view`, to
    /// a window in view now when `in_view_now`.
    pub(super) fn new(in_view: bool, in_view_now: bool) -> Self {
        Self {
            in_view,
            seen_otherwise: in_view_now != in_view,
        }
    }

    /// Notes that the window was seen in view when `in_view_now`, and says
    /// whether the change has landed by then.
    pub(super) fn landed(&mut self, in_view_now: bool) -> bool {
        if in_view_now != self.in_view {
            self.seen_otherwise = true;
        }
        in_view_now == self.in_view && self.seen_otherwise
    }
}

#[cfg(test)]
#[path = "tests/posted_tests.rs"]
mod tests;
