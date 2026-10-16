//! The desktop of the fake [`World`].
//!
//! Commands only place windows, so placements are recorded and can be
//! made to fail. Everything else succeeds and changes nothing, except
//! the frames of transitions: no command posts any, and `doctor` asks
//! with none whether the desktop takes them, which [`World::frames`]
//! answers. By default it does not.

use mosaico_core::config::HidingBehaviour;
use mosaico_core::frames::{Frame, FrameControl, FrameOutcome};
use mosaico_core::{Error, HidingEvents, Rect, Result, WindowControl, WindowId};

use super::World;

/// What asking the fake desktop to move windows frame by frame does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Frames {
    /// It cannot, as a platform without frames says.
    Unsupported,
    /// It can.
    Supported,
    /// The question fails, as on a system that cannot be asked.
    Fails,
}

impl FrameControl for World {
    fn place_frames(&self, frames: &[Frame], outcomes: &mut Vec<FrameOutcome>) -> Result<()> {
        // Commands only ask, with no frames.
        assert!(frames.is_empty(), "a command posted frames: {frames:?}");
        outcomes.clear();
        match self.frames.get() {
            Frames::Unsupported => Err(Error::Unsupported("moving windows frame by frame")),
            Frames::Supported => Ok(()),
            Frames::Fails => Err(Error::platform(
                "move windows frame by frame",
                "access denied",
            )),
        }
    }
}

impl WindowControl for World {
    fn place(&self, window: WindowId, rect: Rect) -> Result<()> {
        if let Some(error) = self.place_error.borrow().clone() {
            return Err(Error::platform("move the window", error));
        }
        self.placed.borrow_mut().push((window, rect));
        Ok(())
    }

    fn pre_place(&self, _: WindowId, _: Rect) -> Result<()> {
        Ok(())
    }

    fn repaint(&self, _: WindowId) -> Result<()> {
        Ok(())
    }

    fn focus(&self, _: WindowId) -> Result<()> {
        Ok(())
    }

    fn put_behind(&self, _: WindowId, _: WindowId) -> Result<()> {
        Ok(())
    }

    fn close(&self, _: WindowId) -> Result<()> {
        Ok(())
    }

    fn minimize(&self, _: WindowId) -> Result<()> {
        Ok(())
    }

    fn hide(&self, _: WindowId, _: HidingBehaviour) -> Result<()> {
        Ok(())
    }

    fn show(&self, _: WindowId, _: HidingBehaviour) -> Result<()> {
        Ok(())
    }

    fn hiding_events(&self, _: HidingBehaviour) -> Option<HidingEvents> {
        Some(HidingEvents::default())
    }

    fn reveal(&self, _: WindowId) -> Result<()> {
        Ok(())
    }

    fn move_cursor(&self, _: i32, _: i32) -> Result<()> {
        Ok(())
    }
}
