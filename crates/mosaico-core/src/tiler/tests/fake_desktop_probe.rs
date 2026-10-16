//! The fake desktop's windows, as the frame table reads and moves them.

use std::collections::HashMap;

use crate::frames::FrameProbe;
use crate::{Rect, WindowId};

use super::super::FakeWindow;
use super::{Queued, land};

/// The fake desktop's windows, as a platform reads and moves them.
pub(super) struct Probe<'a> {
    pub(super) windows: &'a mut HashMap<WindowId, FakeWindow>,
}

impl FrameProbe for Probe<'_> {
    type Kept = ();

    fn exists(&self, window: WindowId) -> bool {
        self.windows.contains_key(&window)
    }

    fn minimized(&self, window: WindowId) -> bool {
        self.windows.get(&window).is_some_and(|w| w.minimized)
    }

    fn maximized(&self, window: WindowId) -> bool {
        self.windows.get(&window).is_some_and(|w| w.maximized)
    }

    fn shown(&self, window: WindowId) -> bool {
        self.windows.get(&window).is_some_and(|w| w.visible)
    }

    fn responding(&self, window: WindowId) -> bool {
        self.windows.get(&window).is_some_and(|w| !w.hung)
    }

    fn at(&self, window: WindowId) -> Option<Rect> {
        self.windows.get(&window).map(|w| w.rect)
    }

    /// The process and the thread that made the window: a new window given
    /// the id of an old one has another.
    fn owner(&self, window: WindowId) -> Option<u64> {
        let owner = |w: &FakeWindow| (u64::from(w.process) << 32) | u64::from(w.thread);
        self.windows.get(&window).map(owner)
    }

    fn keep(&self, window: WindowId) -> Option<()> {
        self.exists(window).then_some(())
    }

    fn aim(&self, _: WindowId, _: &(), rect: Rect) -> Rect {
        rect
    }

    fn visible(&self, _: &(), at: Rect) -> Rect {
        at
    }

    fn post(&mut self, window: WindowId, to: Rect) -> bool {
        let Some(w) = self.windows.get_mut(&window) else {
            return false;
        };
        if w.frame_delay == 0 && w.queued.is_empty() {
            land(w, Queued::Frame(to));
        } else {
            w.queued.push((Queued::Frame(to), w.frame_delay));
        }
        true
    }
}
