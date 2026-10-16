//! A platform that records the window changes it is asked to make.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use crate::config::HidingBehaviour;
use crate::frames::{Frame, FrameControl, FrameOutcome};
use crate::{Error, HidingEvents, Rect, Result, WindowControl, WindowId};

/// A change the platform was asked to make.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum Call {
    Place(WindowId, Rect),
    PrePlace(WindowId, Rect),
    PlaceFrames(Vec<Frame>),
    Repaint(WindowId),
    Focus(WindowId),
    PutBehind(WindowId, WindowId),
    Close(WindowId),
    Minimize(WindowId),
    Hide(WindowId, HidingBehaviour),
    Show(WindowId, HidingBehaviour),
    Reveal(WindowId),
    MoveCursor(i32, i32),
}

/// A platform that records every change it is asked to make.
#[derive(Default)]
pub(super) struct Recorder {
    pub(super) calls: RefCell<Vec<Call>>,
    /// Whether every change fails, as it does for a window that is gone.
    pub(super) failing: Cell<bool>,
    /// The last frame of each window, where it is taken to be.
    reached: RefCell<HashMap<WindowId, Rect>>,
}

impl Recorder {
    fn record(&self, call: Call) -> Result<()> {
        self.calls.borrow_mut().push(call);
        if self.failing.get() {
            return Err(Error::platform("change the window", "it is gone"));
        }
        Ok(())
    }
}

impl WindowControl for Recorder {
    fn place(&self, window: WindowId, rect: Rect) -> Result<()> {
        self.record(Call::Place(window, rect))
    }

    fn pre_place(&self, window: WindowId, rect: Rect) -> Result<()> {
        self.record(Call::PrePlace(window, rect))
    }

    fn repaint(&self, window: WindowId) -> Result<()> {
        self.record(Call::Repaint(window))
    }

    fn focus(&self, window: WindowId) -> Result<()> {
        self.record(Call::Focus(window))
    }

    fn put_behind(&self, window: WindowId, other: WindowId) -> Result<()> {
        self.record(Call::PutBehind(window, other))
    }

    fn close(&self, window: WindowId) -> Result<()> {
        self.record(Call::Close(window))
    }

    fn minimize(&self, window: WindowId) -> Result<()> {
        self.record(Call::Minimize(window))
    }

    fn hide(&self, window: WindowId, how: HidingBehaviour) -> Result<()> {
        self.record(Call::Hide(window, how))
    }

    fn show(&self, window: WindowId, how: HidingBehaviour) -> Result<()> {
        self.record(Call::Show(window, how))
    }

    fn hiding_events(&self, _: HidingBehaviour) -> Option<HidingEvents> {
        Some(HidingEvents::default())
    }

    fn reveal(&self, window: WindowId) -> Result<()> {
        self.record(Call::Reveal(window))
    }

    fn move_cursor(&self, x: i32, y: i32) -> Result<()> {
        self.record(Call::MoveCursor(x, y))
    }
}

impl FrameControl for Recorder {
    /// Reports every frame handled, as if each one landed at once: the
    /// window is where its frame before put it, or at its first frame.
    fn place_frames(&self, frames: &[Frame], outcomes: &mut Vec<FrameOutcome>) -> Result<()> {
        self.record(Call::PlaceFrames(frames.to_vec()))?;
        let mut reached = self.reached.borrow_mut();
        outcomes.clear();
        for frame in frames {
            let before = reached.insert(frame.window, frame.rect);
            outcomes.push(FrameOutcome::Handled {
                reached: before.unwrap_or(frame.rect),
            });
        }
        Ok(())
    }
}
