//! A probe of the frame table tests' own: windows that are where the
//! test puts them, and take the moves posted to them at once, or keep
//! them queued, on a clock the test moves on.

use std::cell::Cell;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::id::wid;
use crate::{Rect, WindowId};

use super::FrameProbe;

/// Where window 1 starts.
pub(super) const START: Rect = Rect {
    x: 100,
    y: 100,
    width: 640,
    height: 480,
};

/// A window as the probe sees it.
#[derive(Debug, Clone, Copy)]
pub(super) struct Window {
    pub(super) at: Rect,
    pub(super) minimized: bool,
    pub(super) maximized: bool,
    pub(super) shown: bool,
    pub(super) responding: bool,
    /// Who owns it: a new window given the id of an old one has another.
    pub(super) owner: u64,
    /// Whether a move posted to it lands at once, or stays queued.
    pub(super) lands: bool,
}

/// Windows the test moves through the table.
pub(super) struct Probe {
    pub(super) windows: HashMap<WindowId, Window>,
    /// The time, which the test moves on.
    pub(super) now: Instant,
    /// The moves posted, in order, refused ones included.
    pub(super) posted: Vec<(WindowId, Rect)>,
    /// How many times a window was read for its motion.
    pub(super) keeps: Cell<u32>,
    /// What a window read for its motion keeps: how far right of each
    /// frame it is put.
    pub(super) offset: i32,
    /// Whether the system refuses every post.
    pub(super) refuses: bool,
    /// The size of the window's pixels, in the probe's terms: a frame
    /// only puts a window on a multiple of it.
    pub(super) pixel: i32,
}

impl Probe {
    /// Window 1 at `START`, with moves that land at once when `lands`.
    pub(super) fn new(lands: bool) -> Self {
        let window = Window {
            at: START,
            minimized: false,
            maximized: false,
            shown: true,
            responding: true,
            owner: 1,
            lands,
        };
        Self {
            windows: HashMap::from([(wid(1), window)]),
            now: Instant::now(),
            posted: Vec::new(),
            keeps: Cell::new(0),
            offset: 0,
            refuses: false,
            pixel: 1,
        }
    }

    pub(super) fn window(&mut self) -> &mut Window {
        self.windows.get_mut(&wid(1)).expect("window 1 is open")
    }

    /// Lets `ms` milliseconds pass.
    pub(super) fn wait(&mut self, ms: u64) {
        self.now += Duration::from_millis(ms);
    }
}

impl FrameProbe for Probe {
    type Kept = i32;

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
        self.windows.get(&window).is_some_and(|w| w.shown)
    }

    fn responding(&self, window: WindowId) -> bool {
        self.windows.get(&window).is_some_and(|w| w.responding)
    }

    fn owner(&self, window: WindowId) -> Option<u64> {
        self.windows.get(&window).map(|w| w.owner)
    }

    fn at(&self, window: WindowId) -> Option<Rect> {
        self.windows.get(&window).map(|w| w.at)
    }

    fn keep(&self, window: WindowId) -> Option<i32> {
        self.keeps.set(self.keeps.get() + 1);
        self.exists(window).then_some(self.offset)
    }

    fn aim(&self, _: WindowId, offset: &i32, rect: Rect) -> Rect {
        let snap = |v: i32| v - v.rem_euclid(self.pixel);
        let (x, y) = (snap(rect.x + offset), snap(rect.y));
        Rect::new(x, y, snap(rect.width), snap(rect.height))
    }

    fn visible(&self, _: &i32, at: Rect) -> Rect {
        at
    }

    fn post(&mut self, window: WindowId, to: Rect) -> bool {
        self.posted.push((window, to));
        if self.refuses {
            return false;
        }
        if let Some(w) = self.windows.get_mut(&window)
            && w.lands
        {
            w.at = to;
        }
        true
    }
}
