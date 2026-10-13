//! Tiler tests against a fake desktop.
//!
//! [`Harness`] owns a tiler and a [`FakeDesktop`] and applies every
//! effect the tiler returns to the fake, the way the platform applier
//! would, so a test can drive several steps and assert on both the
//! effects and the resulting state.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::action::MAX_WORKSPACES;
use crate::config::HidingBehaviour;
use crate::{Action, MonitorKey, Rect, WindowEvent, Workspace};

use super::*;

#[path = "tests/display_tests.rs"]
mod display_tests;
#[path = "tests/empty_focus_tests.rs"]
mod empty_focus_tests;
#[path = "tests/event_tests.rs"]
mod event_tests;
#[path = "tests/focus_tests.rs"]
mod focus_tests;
#[path = "tests/layout_tests.rs"]
mod layout_tests;
#[path = "tests/move_tests.rs"]
mod move_tests;
#[path = "tests/pre_place_tests.rs"]
mod pre_place_tests;
#[path = "tests/work_area_tests.rs"]
mod work_area_tests;
#[path = "tests/workspace_tests.rs"]
mod workspace_tests;

/// Width of every fake monitor.
pub(super) const MON_W: i32 = 1920;
/// Height of every fake monitor.
pub(super) const MON_H: i32 = 1080;

/// A window on the fake desktop.
#[derive(Debug, Clone)]
pub(super) struct FakeWindow {
    pub(super) visible: bool,
    pub(super) minimized: bool,
    pub(super) maximized: bool,
    pub(super) cloaked: bool,
    pub(super) rect: Rect,
    pub(super) owner: Option<usize>,
    pub(super) monitor: usize,
    pub(super) facts: Option<WindowFacts>,
}

impl FakeWindow {
    /// A visible, captioned application window on monitor `monitor`.
    pub(super) fn app(monitor: usize) -> Self {
        Self {
            visible: true,
            minimized: false,
            maximized: false,
            cloaked: false,
            rect: Rect::new(monitor as i32 * MON_W + 100, 100, 640, 480),
            owner: None,
            monitor,
            facts: Some(WindowFacts {
                class: "App".into(),
                title: "app".into(),
                path: "/apps/app".into(),
                has_caption: true,
            }),
        }
    }
}

/// An in-memory desktop.
#[derive(Debug, Default)]
pub(super) struct FakeDesktop {
    pub(super) windows: HashMap<usize, FakeWindow>,
    pub(super) cursor: Option<(i32, i32)>,
    pub(super) focused: Option<usize>,
}

impl Desktop for FakeDesktop {
    fn is_visible(&self, window: usize) -> bool {
        self.windows.get(&window).is_some_and(|w| w.visible)
    }

    fn is_minimized(&self, window: usize) -> bool {
        self.windows.get(&window).is_some_and(|w| w.minimized)
    }

    fn is_maximized(&self, window: usize) -> bool {
        self.windows.get(&window).is_some_and(|w| w.maximized)
    }

    fn rect(&self, window: usize) -> Option<Rect> {
        self.windows.get(&window).map(|w| w.rect)
    }

    fn owner(&self, window: usize) -> Option<usize> {
        self.windows.get(&window).and_then(|w| w.owner)
    }

    fn monitor_of(&self, window: usize) -> usize {
        self.windows.get(&window).map_or(0, |w| w.monitor)
    }

    fn cursor(&self) -> Option<(i32, i32)> {
        self.cursor
    }

    fn candidate(&self, window: usize) -> Option<WindowFacts> {
        self.windows
            .get(&window)
            .filter(|w| !w.cloaked)
            .and_then(|w| w.facts.clone())
    }
}

impl FakeDesktop {
    /// Applies an effect the way a platform would.
    fn apply(&mut self, effect: &Effect) {
        match *effect {
            Effect::Place { window, rect, .. } => {
                if let Some(w) = self.windows.get_mut(&window) {
                    w.rect = rect;
                    w.maximized = false;
                    w.minimized = false;
                }
            }
            Effect::Hide { window, how } => {
                if let Some(w) = self.windows.get_mut(&window) {
                    match how {
                        HidingBehaviour::Cloak => w.cloaked = true,
                        HidingBehaviour::Hide => w.visible = false,
                        HidingBehaviour::Minimize => w.minimized = true,
                    }
                }
            }
            Effect::Show { window, .. } | Effect::Reveal { window } => {
                if let Some(w) = self.windows.get_mut(&window) {
                    w.cloaked = false;
                    w.visible = true;
                    w.minimized = false;
                }
            }
            Effect::Minimize { window } => {
                if let Some(w) = self.windows.get_mut(&window) {
                    w.minimized = true;
                }
            }
            Effect::Focus { window, .. } => self.focused = Some(window),
            _ => {}
        }
    }
}

/// `count` monitors side by side, the first one primary, with ids
/// `0..count` and distinct keys.
pub(super) fn displays(count: usize) -> Vec<Display> {
    (0..count)
        .map(|i| Display {
            id: i,
            key: key(&format!("mon{i}")),
            work_area: Rect::new(i as i32 * MON_W, 0, MON_W, MON_H),
            primary: i == 0,
        })
        .collect()
}

/// A monitor key with only a stable id.
pub(super) fn key(name: &str) -> MonitorKey {
    MonitorKey {
        stable_id: Some(name.into()),
        ..Default::default()
    }
}

/// A tiler on a fake desktop that applies its own effects.
pub(super) struct Harness {
    pub(super) tiler: Tiler,
    pub(super) desktop: FakeDesktop,
    pub(super) now: Instant,
    /// Effects of the latest step.
    pub(super) effects: Vec<Effect>,
}

impl Harness {
    /// Starts a tiler on `monitors` monitors with the given windows,
    /// each `(id, monitor)`, already open.
    pub(super) fn new(monitors: usize, windows: &[(usize, usize)]) -> Self {
        Self::with_settings(Settings::default(), monitors, windows)
    }

    pub(super) fn with_settings(
        settings: Settings,
        monitors: usize,
        windows: &[(usize, usize)],
    ) -> Self {
        let mut desktop = FakeDesktop::default();
        for &(id, mon) in windows {
            desktop.windows.insert(id, FakeWindow::app(mon));
        }
        let ids: Vec<usize> = windows.iter().map(|&(id, _)| id).collect();
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
            self.desktop.apply(effect);
        }
        self.effects = effects;
    }

    /// Runs one input and applies its effects.
    pub(super) fn step(&mut self, input: Input) -> &[Effect] {
        let effects = self.tiler.step(input, &self.desktop, self.now);
        self.apply(effects);
        &self.effects
    }

    pub(super) fn action(&mut self, action: Action) -> &[Effect] {
        self.step(Input::Action(action))
    }

    pub(super) fn event(&mut self, event: WindowEvent) -> &[Effect] {
        self.step(Input::Event(event))
    }

    /// Opens a new visible window on `monitor` and reports it.
    pub(super) fn open(&mut self, id: usize, monitor: usize) -> &[Effect] {
        self.desktop.windows.insert(id, FakeWindow::app(monitor));
        self.event(WindowEvent::Created { hwnd: id })
    }

    pub(super) fn advance(&mut self, by: Duration) {
        self.now += by;
    }

    /// Windows on the active workspace of monitor `mon`, in order.
    pub(super) fn active(&self, mon: usize) -> Vec<usize> {
        self.tiler.monitors()[mon].active().handles().to_vec()
    }

    pub(super) fn workspace(&self, mon: usize, ws: usize) -> &Workspace {
        &self.tiler.monitors()[mon].workspaces()[ws]
    }

    pub(super) fn window(&mut self, window: usize) -> &mut FakeWindow {
        self.desktop
            .windows
            .get_mut(&window)
            .expect("window exists")
    }

    pub(super) fn rect_of(&self, window: usize) -> Rect {
        self.desktop.windows[&window].rect
    }

    /// The window the last step asked to focus, if it asked.
    pub(super) fn focus_request(&self) -> Option<usize> {
        self.effects.iter().rev().find_map(|e| match e {
            Effect::Focus { window, .. } => Some(*window),
            _ => None,
        })
    }

    /// Placements of the last step, as `(window, rect, timing)`.
    pub(super) fn placements(&self) -> Vec<(usize, Rect, Timing)> {
        self.effects
            .iter()
            .filter_map(|e| match *e {
                Effect::Place {
                    window,
                    rect,
                    timing,
                } => Some((window, rect, timing)),
                _ => None,
            })
            .collect()
    }
}

/// Bare monitor state for tests of the free functions.
pub(super) fn make_monitor(id: usize) -> MonitorState {
    MonitorState {
        id,
        key: MonitorKey::default(),
        os_work_area: Rect::new(0, 0, MON_W, MON_H),
        work_area: Rect::new(0, 0, MON_W, MON_H),
        workspaces: (0..MAX_WORKSPACES).map(|_| Workspace::new()).collect(),
        active_workspace: 0,
    }
}

pub(super) fn make_monitors(n: usize) -> Vec<MonitorState> {
    (0..n).map(make_monitor).collect()
}

/// Runs the real display change rebuild with unidentified monitors, the
/// first of which is primary.
pub(super) fn simulate_display_change(
    old_monitors: &mut Vec<MonitorState>,
    new_infos: Vec<(usize, Rect)>,
) -> Vec<MonitorState> {
    let infos: Vec<Display> = new_infos
        .into_iter()
        .enumerate()
        .map(|(i, (id, work_area))| Display {
            id,
            key: MonitorKey::default(),
            work_area,
            primary: i == 0,
        })
        .collect();
    let layouts = vec![LayoutKind::default(); MAX_WORKSPACES as usize];
    let mut memory = reconnect::ReconnectMemory::default();
    display_rebuild::rebuild_monitors(std::mem::take(old_monitors), &infos, &layouts, &mut memory)
        .monitors
}
