//! Keeping a window's other changes in order with the frames a
//! [`FrameTable`] posted to it.

use std::time::Instant;

use crate::WindowId;

use super::super::FrameProbe;
use super::super::posted::ViewNote;
use super::kept::Kept;
use super::{ASYNC_GRACE, FrameTable};

impl<K: Copy> FrameTable<K> {
    /// Whether a change made to `window` at `now` is posted behind what
    /// may be queued for the window instead of made at once. It is for
    /// [`ASYNC_GRACE`] after anything was posted to the window; after
    /// that, for as long as the window has not been seen to move since the
    /// last thing posted to it, a frame or any other change, which may
    /// still be queued; and, once it was given a frame, while its
    /// application does not respond, since a change made at once would
    /// wait for it. A window never given a frame, or given none since
    /// [`settle_responding`](Self::settle_responding) forgot it, is changed
    /// at once, whether or not its application responds.
    ///
    /// A change posted lands after everything posted to the window before
    /// it. Past the grace, any move seen ends the order, whoever made it,
    /// and a change made at once lands before anything still queued: the
    /// later of two frames queued, when the application took the earlier
    /// and stopped before it was reported not responding, or what was
    /// queued when something else moved the window. Unless something is
    /// posted to the window in between, the answer only changes from
    /// posted to made at once, as time passes, once the window is seen to
    /// move, or once its application responds again. It changes the other
    /// way in two cases: its application stops responding, or a frame is
    /// given to the window while its application does not respond, which
    /// posts nothing but starts what the table keeps for it. Asked right
    /// after a change, `true` means it was posted, as a post renews the
    /// grace, or that the application stopped responding since.
    pub fn posts_behind<P>(&mut self, window: WindowId, probe: &P, now: Instant) -> bool
    where
        P: FrameProbe<Kept = K>,
    {
        let Some(owner) = probe.owner(window) else {
            return false;
        };
        let Some(kept) = self.known(window, owner) else {
            return false;
        };
        kept.seen_in_view(in_view_now(window, probe));
        let since = |at: Instant| now.saturating_duration_since(at);
        if kept.post.is_some_and(|post| since(post.when) < ASYNC_GRACE) {
            return true;
        }
        if let Some(at) = probe.at(window) {
            kept.seen_at(at);
        }
        kept.post.is_some_and(|post| post.unmoved()) || !probe.responding(window)
    }

    /// Notes that a change was posted to `window` at `now`, with where the
    /// window was then, so the changes after it are posted behind it too.
    pub fn posted<P>(&mut self, window: WindowId, probe: &P, now: Instant)
    where
        P: FrameProbe<Kept = K>,
    {
        if let Some(owner) = probe.owner(window)
            && let Some(at) = probe.at(window)
        {
            self.owned(window, owner).posted(at, now);
        }
    }

    /// Notes that a change of `window`'s show state was posted to it at
    /// `now`, as [`posted`](Self::posted) does, and that it leaves the
    /// window in view when `in_view`, for what is posted after it.
    pub fn posted_show_state<P>(&mut self, window: WindowId, probe: &P, now: Instant, in_view: bool)
    where
        P: FrameProbe<Kept = K>,
    {
        if let Some(owner) = probe.owner(window)
            && let Some(at) = probe.at(window)
        {
            let in_view_now = in_view_now(window, probe);
            let kept = self.owned(window, owner);
            kept.posted(at, now);
            kept.in_view = Some(ViewNote::new(in_view, in_view_now));
        }
    }

    /// Notes that a change of `window`'s show state was made at once, so
    /// the window now is as it leaves it.
    pub fn show_state_made<P>(&mut self, window: WindowId, probe: &P)
    where
        P: FrameProbe<Kept = K>,
    {
        if let Some(owner) = probe.owner(window)
            && let Some(kept) = self.known(window, owner)
        {
            kept.in_view = None;
        }
    }

    /// Whether `window` is to be in view, shown and not minimized, once
    /// what was posted to it lands, as the last change of its show state
    /// posted to it leaves it. `None` when none was posted, or one was made
    /// at once after it, or the window has been seen as that change leaves
    /// it, by this call or by [`posts_behind`](Self::posts_behind): the
    /// window is then as it is to be, whatever was posted.
    pub fn in_view_once_landed<P>(&mut self, window: WindowId, probe: &P) -> Option<bool>
    where
        P: FrameProbe<Kept = K>,
    {
        let owner = probe.owner(window)?;
        let kept = self.known(window, owner)?;
        kept.seen_in_view(in_view_now(window, probe));
        kept.in_view.map(|note| note.in_view)
    }

    /// Forgets every window something was last posted to more than
    /// [`ASYNC_GRACE`] before `now`, or nothing, whose application
    /// responds, so that the changes made to it are made at once until it
    /// is given a frame again (see `FrameControl::settle_responding`).
    pub fn settle_responding<P>(&mut self, probe: &P, now: Instant)
    where
        P: FrameProbe<Kept = K>,
    {
        let recent = |kept: &Kept<K>| {
            (kept.post).is_some_and(|post| now.saturating_duration_since(post.when) < ASYNC_GRACE)
        };
        self.windows
            .retain(|&window, kept| recent(kept) || !probe.responding(window));
    }
}

/// Whether `window` is in view now: shown, and not minimized.
fn in_view_now<P: FrameProbe>(window: WindowId, probe: &P) -> bool {
    probe.shown(window) && !probe.minimized(window)
}
