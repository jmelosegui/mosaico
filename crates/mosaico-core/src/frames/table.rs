//! What a platform keeps about the frames it posts, and what it decides
//! became of each.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::WindowId;

use super::{Frame, FrameOutcome, FrameProbe};

#[path = "table_kept.rs"]
mod kept;
#[path = "table_queue.rs"]
mod queue;

use kept::Kept;

/// How long a frame posted to a window that has not been seen to move
/// since holds the next one back. A slow application gets at most one
/// frame this often, never a backlog that grows with the frame rate, and
/// a window that takes its frames without moving, as one that rounds its
/// size or puts it back does, gets them at this pace too.
pub const PENDING_LIMIT: Duration = Duration::from_millis(250);

/// How long after something was posted to a window the changes made to
/// it are posted too, behind it, rather than made at once, even once it
/// has been seen to move since (see [`FrameTable::posts_behind`]). About
/// as long as the system gives an application that is not handling its
/// messages before it calls it not responding.
pub const ASYNC_GRACE: Duration = Duration::from_secs(5);

/// What a platform keeps about the frames it posts to its windows, and
/// how it decides what became of each, the same way on every platform.
///
/// The platform reads its windows and posts their moves through a
/// [`FrameProbe`]. For each frame, in this order: a window that is gone
/// is gone; a minimized or a maximized one is left alone; one that
/// another owner has now, and whose motion is not starting over, is gone;
/// the first frame of a motion reads the window again; a window whose
/// application is not responding is pending; a frame posted less than
/// [`PENDING_LIMIT`] ago to a window not seen to move since holds the new
/// one back; a frame the window can only take at the rect it is at is not
/// needed; and any other is posted. A post the system refuses leaves
/// nothing behind.
///
/// It also says whether a window's other changes are posted behind what
/// was posted to it ([`posts_behind`](Self::posts_behind)), keeps what
/// such a change needs to know ([`kept_for`](Self::kept_for),
/// [`in_view_once_landed`](Self::in_view_once_landed)), and forgets the
/// windows that need it no longer ([`settle_responding`](Self::settle_responding)).
#[derive(Debug)]
pub struct FrameTable<K> {
    windows: HashMap<WindowId, Kept<K>>,
}

impl<K> Default for FrameTable<K> {
    fn default() -> Self {
        Self {
            windows: HashMap::new(),
        }
    }
}

impl<K: Copy> FrameTable<K> {
    /// Posts each window its frame through `probe`, unless the window
    /// cannot take it, after clearing `outcomes`, and puts there what was
    /// found of each window at `now`: one [`FrameOutcome`] per frame, in
    /// order.
    pub fn place<P>(
        &mut self,
        frames: &[Frame],
        probe: &mut P,
        now: Instant,
        outcomes: &mut Vec<FrameOutcome>,
    ) where
        P: FrameProbe<Kept = K>,
    {
        // Nothing is left to keep about a window that is gone.
        self.windows.retain(|&window, _| {
            frames.iter().any(|frame| frame.window == window) || probe.exists(window)
        });
        outcomes.clear();
        for frame in frames {
            let outcome = self.take(frame, probe, now);
            outcomes.push(outcome);
        }
    }

    fn take<P: FrameProbe<Kept = K>>(
        &mut self,
        frame: &Frame,
        probe: &mut P,
        now: Instant,
    ) -> FrameOutcome {
        let window = frame.window;
        if !probe.exists(window) {
            self.windows.remove(&window);
            return FrameOutcome::Gone;
        }
        if probe.minimized(window) {
            return FrameOutcome::Minimized;
        }
        if probe.maximized(window) {
            return FrameOutcome::Maximized;
        }
        let Some(kept) = self.kept(window, frame.first, probe) else {
            return FrameOutcome::Gone;
        };
        let Some(platform) = kept.platform else {
            return FrameOutcome::Gone;
        };
        if !probe.responding(window) {
            return FrameOutcome::Pending;
        }
        let Some(at) = probe.at(window) else {
            return FrameOutcome::Gone;
        };
        kept.seen_at(at);
        if kept.frame.is_some_and(|last| last.holds_back(now)) {
            return FrameOutcome::Pending;
        }
        let to = probe.aim(window, &platform, frame.rect);
        if to != at && probe.post(window, to) {
            kept.posted_frame(at, now);
        }
        FrameOutcome::Handled {
            reached: probe.visible(&platform, at),
        }
    }
}

#[cfg(test)]
#[path = "tests/table_tests.rs"]
mod tests;
