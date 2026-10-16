//! A tiler on a fake desktop.
//!
//! [`Harness`] owns a tiler and a [`FakeDesktop`] and applies every
//! effect the tiler returns to the fake straight away, so a test can
//! drive several steps and assert on both the effects and the
//! resulting state. Nothing is held back and no echo is dropped: tests
//! of that run the tiling manager on a `FakePlatform`.

use std::time::{Duration, Instant};

use crate::id::wid;
use crate::{Action, Rect, WindowEvent, WindowId, Workspace};

use super::super::{Effect, Input, Settings, Tiler, Timing};
use super::{FakeDesktop, FakeWindow, displays};

/// A tiler on a fake desktop that applies its own effects.
pub(in crate::tiler) struct Harness {
    pub(in crate::tiler) tiler: Tiler,
    pub(in crate::tiler) desktop: FakeDesktop,
    pub(in crate::tiler) now: Instant,
    /// Effects of the latest step.
    pub(in crate::tiler) effects: Vec<Effect>,
}

impl Harness {
    /// Starts a tiler on `monitors` monitors with the given windows,
    /// each `(id, monitor)`, already open.
    pub(in crate::tiler) fn new(monitors: usize, windows: &[(u64, usize)]) -> Self {
        Self::with_settings(Settings::default(), monitors, windows)
    }

    pub(in crate::tiler) fn with_settings(
        settings: Settings,
        monitors: usize,
        windows: &[(u64, usize)],
    ) -> Self {
        let mut desktop = FakeDesktop::default();
        for &(id, mon) in windows {
            desktop.windows.insert(wid(id), FakeWindow::app(mon));
            desktop.raise(wid(id));
        }
        let ids: Vec<WindowId> = windows.iter().map(|&(id, _)| wid(id)).collect();
        let now = Instant::now();
        let (tiler, effects) = Tiler::new(settings, &displays(monitors), &ids, &desktop, now);
        let mut harness = Self {
            tiler,
            desktop,
            now,
            effects: Vec::new(),
        };
        harness.apply(effects);
        harness
    }

    fn apply(&mut self, effects: Vec<Effect>) {
        for effect in &effects {
            self.desktop.apply(effect, self.now);
        }
        self.effects = effects;
    }

    /// Runs one input and applies its effects.
    pub(in crate::tiler) fn step(&mut self, input: Input) -> &[Effect] {
        let effects = self.tiler.step(input, &self.desktop, self.now);
        self.apply(effects);
        &self.effects
    }

    pub(in crate::tiler) fn action(&mut self, action: Action) -> &[Effect] {
        self.step(Input::Action(action))
    }

    /// Reports a window event. A window reported focused is in front,
    /// whoever focused it.
    pub(in crate::tiler) fn event(&mut self, event: WindowEvent) -> &[Effect] {
        if let WindowEvent::Focused { window } = event {
            self.desktop.raise(window);
        }
        self.step(Input::Event(event))
    }

    /// Opens a new visible window on `monitor` and reports it.
    pub(in crate::tiler) fn open(&mut self, id: u64, monitor: usize) -> &[Effect] {
        self.desktop
            .windows
            .insert(wid(id), FakeWindow::app(monitor));
        self.desktop.raise(wid(id));
        self.event(WindowEvent::Created { window: wid(id) })
    }

    pub(in crate::tiler) fn advance(&mut self, by: Duration) {
        self.now += by;
    }

    /// Windows on the active workspace of monitor `mon`, in order.
    pub(in crate::tiler) fn active(&self, mon: usize) -> Vec<u64> {
        self.tiler.monitors()[mon]
            .active()
            .windows()
            .iter()
            .map(|w| w.raw())
            .collect()
    }

    pub(in crate::tiler) fn workspace(&self, mon: usize, ws: usize) -> &Workspace {
        &self.tiler.monitors()[mon].workspaces()[ws]
    }

    pub(in crate::tiler) fn window(&mut self, window: u64) -> &mut FakeWindow {
        self.desktop
            .windows
            .get_mut(&wid(window))
            .expect("window exists")
    }

    pub(in crate::tiler) fn rect_of(&self, window: u64) -> Rect {
        self.desktop.windows[&wid(window)].rect
    }

    /// The window the last step asked to focus, if it asked.
    pub(in crate::tiler) fn focus_request(&self) -> Option<u64> {
        self.effects.iter().rev().find_map(|e| match e {
            Effect::Focus { window, .. } => Some(window.raw()),
            _ => None,
        })
    }

    /// Placements of the last step, as `(window, rect, timing)`.
    pub(in crate::tiler) fn placements(&self) -> Vec<(u64, Rect, Timing)> {
        self.effects
            .iter()
            .filter_map(|e| match *e {
                Effect::Place {
                    window,
                    rect,
                    timing,
                    ..
                } => Some((window.raw(), rect, timing)),
                _ => None,
            })
            .collect()
    }
}
