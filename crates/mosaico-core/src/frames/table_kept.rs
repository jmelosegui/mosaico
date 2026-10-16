//! What a [`FrameTable`] keeps about one window, looked up with the
//! window's owner every time.

use std::time::Instant;

use crate::{Rect, WindowId};

use super::super::FrameProbe;
use super::super::posted::{LastPost, ViewNote};
use super::FrameTable;

/// What the table keeps about one window.
#[derive(Debug, Clone, Copy)]
pub(super) struct Kept<K> {
    /// Who owns the window, as the platform tells them apart.
    owner: u64,
    /// What the platform keeps for the window's motion, read with its
    /// first frame.
    pub(super) platform: Option<K>,
    /// The last frame posted to the window, which may hold the next back.
    pub(super) frame: Option<LastPost>,
    /// The last thing posted to the window, a frame or any other change,
    /// which the changes after it keep their order with.
    pub(super) post: Option<LastPost>,
    /// The last change of the window's show state posted to it. `None`
    /// when none was posted, one was made at once after it, or the window
    /// has been seen as it leaves it, once seen otherwise.
    pub(super) in_view: Option<ViewNote>,
}

impl<K> Kept<K> {
    fn new(owner: u64) -> Self {
        Self {
            owner,
            platform: None,
            frame: None,
            post: None,
            in_view: None,
        }
    }

    /// Notes that a frame was posted at `now` to the window at `at`.
    pub(super) fn posted_frame(&mut self, at: Rect, now: Instant) {
        self.frame = Some(LastPost::new(now, at));
        self.post = self.frame;
    }

    /// Notes that a change other than a frame was posted at `now` to the
    /// window at `at`.
    pub(super) fn posted(&mut self, at: Rect, now: Instant) {
        self.post = Some(LastPost::new(now, at));
    }

    /// Notes that the window was seen at `at`.
    pub(super) fn seen_at(&mut self, at: Rect) {
        for last in [&mut self.frame, &mut self.post].into_iter().flatten() {
            last.seen_at(at);
        }
    }

    /// Notes that the window was seen in view when `in_view`. Once the
    /// last change of its show state posted has landed, the window shows
    /// what it is to be, and the note of that change goes.
    pub(super) fn seen_in_view(&mut self, in_view: bool) {
        if self
            .in_view
            .as_mut()
            .is_some_and(|note| note.landed(in_view))
        {
            self.in_view = None;
        }
    }
}

impl<K: Copy> FrameTable<K> {
    /// What is kept about `window` for a frame, the first of its motion
    /// when `first`, read again then. A window whose owner changed starts
    /// over with the first frame of its motion; for any other it is
    /// another window, and `None`, as it is once the window is gone.
    pub(super) fn kept<P>(
        &mut self,
        window: WindowId,
        first: bool,
        probe: &P,
    ) -> Option<&mut Kept<K>>
    where
        P: FrameProbe<Kept = K>,
    {
        let owner = probe.owner(window)?;
        let known = self.windows.get(&window).map(|kept| kept.owner);
        if known.is_some_and(|known| known != owner) && !first {
            self.windows.remove(&window);
            return None;
        }
        let kept = self.owned(window, owner);
        if first || kept.platform.is_none() {
            kept.platform = Some(probe.keep(window)?);
        }
        Some(kept)
    }

    /// What the platform kept about `window` for its last motion, read
    /// with a frame of it while the window was neither minimized nor
    /// maximized. `None` when no motion of it has started, or another
    /// owner has it now.
    pub fn kept_for<P>(&mut self, window: WindowId, probe: &P) -> Option<K>
    where
        P: FrameProbe<Kept = K>,
    {
        let owner = probe.owner(window)?;
        self.known(window, owner)?.platform
    }

    /// What is kept about `window`, owned by `owner`, made anew when it is
    /// missing or was another owner's.
    pub(super) fn owned(&mut self, window: WindowId, owner: u64) -> &mut Kept<K> {
        let kept = self
            .windows
            .entry(window)
            .or_insert_with(|| Kept::new(owner));
        if kept.owner != owner {
            *kept = Kept::new(owner);
        }
        kept
    }

    /// What is kept about `window` while `owner` still has it. What
    /// another owner left is dropped.
    pub(super) fn known(&mut self, window: WindowId, owner: u64) -> Option<&mut Kept<K>> {
        if self
            .windows
            .get(&window)
            .is_some_and(|kept| kept.owner != owner)
        {
            self.windows.remove(&window);
        }
        self.windows.get_mut(&window)
    }
}
