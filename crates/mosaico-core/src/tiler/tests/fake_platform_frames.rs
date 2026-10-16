//! What a [`FakePlatform`] does for transitions: it moves windows to
//! their frames on its desktop, late for a window whose application is
//! slow, and says when it does that for other changes too (see
//! `fake_desktop_frames.rs`), unless it is made a platform that cannot.

use crate::frames::{Frame, FrameControl, FrameOutcome};
use crate::id::wid;
use crate::tiler::tests::fake_desktop::Queued;
use crate::{Error, Result, WindowId};

use super::{Call, FakePlatform};

#[path = "transition_fake_tests.rs"]
mod tests;

/// How `place_frames` fails on a fake platform, if it does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FrameFault {
    /// It works.
    Works,
    /// It is not supported, as on a platform without frames.
    Refused,
    /// It fails for this many more calls, as it may on a platform that has
    /// frames.
    Broken(u32),
}

impl FakePlatform {
    /// Makes this a platform that cannot move windows frame by frame.
    pub(crate) fn refuse_frames(&self) {
        self.shared.frame_fault.set(FrameFault::Refused);
    }

    /// Makes the next `calls` calls of `place_frames` fail.
    pub(crate) fn break_frames(&self, calls: u32) {
        self.shared.frame_fault.set(FrameFault::Broken(calls));
    }
}

impl FrameControl for FakePlatform {
    fn place_frames(&self, frames: &[Frame], outcomes: &mut Vec<FrameOutcome>) -> Result<()> {
        self.record(Call::PlaceFrames(frames.to_vec()));
        let now = self.shared.now.get();
        match self.shared.frame_fault.get() {
            FrameFault::Refused => {
                return Err(Error::Unsupported("moving windows frame by frame"));
            }
            FrameFault::Broken(left) if left > 0 => {
                self.shared.frame_fault.set(FrameFault::Broken(left - 1));
                let source = "the fake broke its frames".into();
                let operation = "move windows frame by frame";
                return Err(Error::Platform { operation, source });
            }
            _ => {}
        }
        let mut desktop = self.shared.desktop.borrow_mut();
        desktop.take_frames(frames, now, outcomes);
        Ok(())
    }

    fn posts_behind(&self, window: WindowId) -> bool {
        let now = self.shared.now.get();
        self.shared.desktop.borrow_mut().posts_behind(window, now)
    }

    fn settle_responding(&self) {
        self.record(Call::SettleResponding);
        let now = self.shared.now.get();
        self.shared.desktop.borrow_mut().settle_responding(now);
    }
}

impl FakePlatform {
    /// The application of window `id` gets to everything queued for it,
    /// in order, as one busy with something else does once it is done.
    pub(crate) fn catch_up(&self, id: u64) {
        self.shared.desktop.borrow_mut().catch_up(wid(id));
    }

    /// The application of window `id` gets to what is queued for it, in
    /// order, up to and including the first `until`.
    pub(crate) fn catch_up_through(&self, id: u64, until: Queued) {
        let mut desktop = self.shared.desktop.borrow_mut();
        desktop.catch_up_through(wid(id), until);
    }

    /// The application of every window that responds and is not slow gets
    /// to what is queued for it, as it does soon after its frames stop.
    /// Gives the windows that moved, in the order of their ids: the ones a
    /// platform reports, as one that is not moved reports nothing.
    pub(crate) fn catch_up_prompt(&self) -> Vec<WindowId> {
        let mut desktop = self.shared.desktop.borrow_mut();
        let mut windows: Vec<_> = (desktop.windows.iter())
            .filter(|(_, w)| !w.hung && w.frame_delay == 0 && !w.queued.is_empty())
            .map(|(&window, w)| (window, w.rect))
            .collect();
        windows.sort_by_key(|(window, _)| window.raw());
        windows.retain(|&(window, before)| {
            desktop.catch_up(window);
            desktop
                .windows
                .get(&window)
                .is_some_and(|w| w.rect != before)
        });
        windows.into_iter().map(|(window, _)| window).collect()
    }
}
