//! A window of the fake desktop.

use std::sync::atomic::{AtomicU32, Ordering};

use crate::tiler::WindowFacts;
use crate::{Rect, WindowId};

use super::super::MON_W;
use super::Queued;

/// A window on the fake desktop.
#[derive(Debug, Clone)]
pub(crate) struct FakeWindow {
    pub(crate) visible: bool,
    pub(crate) minimized: bool,
    pub(crate) maximized: bool,
    pub(crate) cloaked: bool,
    pub(crate) rect: Rect,
    /// Where a restore puts the window back once it is no longer
    /// maximized: where it last was in its normal state.
    pub(crate) normal: Rect,
    pub(crate) owner: Option<WindowId>,
    pub(crate) monitor: usize,
    pub(crate) facts: Option<WindowFacts>,
    pub(crate) process: u32,
    /// The thread that made it: every window made has its own.
    pub(crate) thread: u32,
    /// How many calls of `place_frames` what is queued for the window
    /// waits before its application gets to it, as for an application
    /// slow to handle it: frames, and the placements and changes of its
    /// show state queued behind them. With 0 a frame lands at once.
    pub(crate) frame_delay: usize,
    /// Whether the window takes the frames posted to it without moving,
    /// as one does with a step smaller than its pixels.
    pub(crate) absorbs_frames: bool,
    /// Whether its application is not responding, so it gets to nothing
    /// queued for it.
    pub(crate) hung: bool,
    /// Whether its application stops responding as the next placement of
    /// the window is made, before the placement gets to it.
    pub(crate) hangs_on_place: bool,
    /// What is queued for the window, oldest first, and for how many
    /// more calls.
    pub(crate) queued: Vec<(Queued, usize)>,
}

impl FakeWindow {
    /// A visible, captioned application window on monitor `monitor`.
    pub(crate) fn app(monitor: usize) -> Self {
        static THREADS: AtomicU32 = AtomicU32::new(1);
        let rect = Rect::new(monitor as i32 * MON_W + 100, 100, 640, 480);
        Self {
            visible: true,
            minimized: false,
            maximized: false,
            cloaked: false,
            rect,
            normal: rect,
            owner: None,
            monitor,
            facts: Some(WindowFacts {
                class: "App".into(),
                title: "app".into(),
                path: "/apps/app".into(),
                has_caption: true,
            }),
            process: 1000,
            thread: THREADS.fetch_add(1, Ordering::Relaxed),
            frame_delay: 0,
            absorbs_frames: false,
            hung: false,
            hangs_on_place: false,
            queued: Vec::new(),
        }
    }
}
