//! What a [`FakePlatform`] does for transitions: it moves windows to
//! their frames on its desktop, late for a window whose application is
//! slow, and says when it does that for other changes too (see
//! `fake_desktop_frames.rs`).

use crate::frames::{Frame, FrameControl, FrameOutcome};
use crate::id::wid;
use crate::{Result, WindowId};

use super::{Call, FakePlatform};

#[path = "transition_fake_tests.rs"]
mod tests;

impl FrameControl for FakePlatform {
    fn place_frames(&self, frames: &[Frame], outcomes: &mut Vec<FrameOutcome>) -> Result<()> {
        self.record(Call::PlaceFrames(frames.to_vec()));
        let now = self.shared.now.get();
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
}
