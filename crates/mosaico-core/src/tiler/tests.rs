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
use crate::id::wid;
use crate::{Action, MonitorId, MonitorKey, Rect, WindowEvent, WindowId, Workspace};

use super::*;

#[path = "tests/border_tests.rs"]
mod border_tests;
#[path = "tests/corner_tests.rs"]
mod corner_tests;
#[path = "tests/corner_tracking_tests.rs"]
mod corner_tracking_tests;
#[path = "tests/decoration_effects_tests.rs"]
mod decoration_effects_tests;
#[path = "tests/display_tests.rs"]
mod display_tests;
#[path = "tests/empty_focus_tests.rs"]
mod empty_focus_tests;
#[path = "tests/event_tests.rs"]
mod event_tests;
#[path = "tests/fallback_tests.rs"]
mod fallback_tests;
#[path = "tests/focus_tests.rs"]
mod focus_tests;
#[path = "tests/global_lineup_tests.rs"]
mod global_lineup_tests;
#[path = "tests/hidden_how_tests.rs"]
mod hidden_how_tests;
#[path = "tests/hiding_change_tests.rs"]
mod hiding_change_tests;
#[path = "tests/layout_tests.rs"]
mod layout_tests;
#[path = "tests/monitor_return_tests.rs"]
mod monitor_return_tests;
#[path = "tests/move_tests.rs"]
mod move_tests;
#[path = "tests/pipeline_tests.rs"]
mod pipeline_tests;
#[path = "tests/pre_place_tests.rs"]
mod pre_place_tests;
#[path = "tests/rules_release_tests.rs"]
mod rules_release_tests;
#[path = "tests/rules_reload_tests.rs"]
mod rules_reload_tests;
#[path = "tests/session_tests.rs"]
mod session_tests;
#[path = "tests/window_effects_tests.rs"]
mod window_effects_tests;
#[path = "tests/window_recorder.rs"]
mod window_recorder;
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
    pub(super) owner: Option<WindowId>,
    pub(super) monitor: usize,
    pub(super) facts: Option<WindowFacts>,
    pub(super) process: u32,
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
            process: 1000,
        }
    }
}

/// An in-memory desktop.
#[derive(Debug, Default)]
pub(super) struct FakeDesktop {
    pub(super) windows: HashMap<WindowId, FakeWindow>,
    pub(super) cursor: Option<(i32, i32)>,
    pub(super) focused: Option<WindowId>,
    /// Border overlays, as the platform keeps them: one per window,
    /// created when first drawn and destroyed by `DropBorder`. The value
    /// is whether the overlay is showing.
    pub(super) borders: HashMap<WindowId, bool>,
}

impl Desktop for FakeDesktop {
    fn is_visible(&self, window: WindowId) -> bool {
        self.windows.get(&window).is_some_and(|w| w.visible)
    }

    fn is_minimized(&self, window: WindowId) -> bool {
        self.windows.get(&window).is_some_and(|w| w.minimized)
    }

    fn is_maximized(&self, window: WindowId) -> bool {
        self.windows.get(&window).is_some_and(|w| w.maximized)
    }

    fn rect(&self, window: WindowId) -> Option<Rect> {
        self.windows.get(&window).map(|w| w.rect)
    }

    fn owner(&self, window: WindowId) -> Option<WindowId> {
        self.windows.get(&window).and_then(|w| w.owner)
    }

    fn identity(&self, window: WindowId) -> Option<WindowIdentity> {
        self.windows.get(&window).map(|w| WindowIdentity {
            process: w.process,
            class: w
                .facts
                .as_ref()
                .map(|f| f.class.clone())
                .unwrap_or_default(),
        })
    }

    fn monitor_of(&self, window: WindowId) -> MonitorId {
        MonitorId::from_raw(self.windows.get(&window).map_or(0, |w| w.monitor as u64))
    }

    fn cursor(&self) -> Option<(i32, i32)> {
        self.cursor
    }

    fn candidate(&self, window: WindowId) -> Option<WindowFacts> {
        self.windows
            .get(&window)
            .filter(|w| !w.cloaked)
            .and_then(|w| w.facts.clone())
    }

    fn facts(&self, window: WindowId) -> Option<WindowFacts> {
        self.windows.get(&window).and_then(|w| w.facts.clone())
    }
}

impl FakeDesktop {
    /// Applies an effect the way a platform would.
    fn apply(&mut self, effect: &Effect) {
        match *effect {
            Effect::Place { window, rect, .. } => {
                // As `WindowControl::place` promises: no longer maximized,
                // but a minimized window stays minimized.
                if let Some(w) = self.windows.get_mut(&window) {
                    w.rect = rect;
                    w.maximized = false;
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
            Effect::Border {
                window,
                style: Some(_),
            } => {
                self.borders.insert(window, true);
            }
            Effect::Border {
                window,
                style: None,
            } => {
                if let Some(showing) = self.borders.get_mut(&window) {
                    *showing = false;
                }
            }
            Effect::DropBorder { window } => {
                self.borders.remove(&window);
            }
            _ => {}
        }
    }
}

/// The platform side of the effect pipeline: what the applier does
/// between the tiler and the desktop.
#[derive(Debug)]
pub(super) struct Pipeline {
    pub(super) coalescer: Coalescer,
    pub(super) expectations: Expectations,
}

impl Pipeline {
    /// A pipeline on a platform whose ways of hiding are `hiding`.
    fn new(hiding: HidingSupport) -> Self {
        Self {
            coalescer: Coalescer::default(),
            expectations: Expectations::new(hiding),
        }
    }
}

/// Applies one effect to the desktop and records it in the hidden-window
/// record, as the platform applier does for every effect it runs.
fn deliver(desktop: &mut FakeDesktop, hidden: &mut HiddenWindows, effect: &Effect) {
    desktop.apply(effect);
    hidden.applied(effect, |w| desktop.identity(w));
}

/// `count` monitors side by side, the first one primary, with ids
/// `0..count` and distinct keys.
pub(super) fn displays(count: usize) -> Vec<Display> {
    (0..count)
        .map(|i| Display {
            id: MonitorId::from_raw(i as u64),
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
    /// What the platform records of windows hidden by switching.
    pub(super) hidden: HiddenWindows,
    settings: Settings,
    monitors: usize,
    /// When set, effects go through the coalescer and the echo filter
    /// the way the platform applier sends them, instead of straight to
    /// the desktop. See [`Harness::through_platform`].
    pub(super) pipeline: Option<Pipeline>,
}

impl Harness {
    /// Starts a tiler on `monitors` monitors with the given windows,
    /// each `(id, monitor)`, already open.
    pub(super) fn new(monitors: usize, windows: &[(u64, usize)]) -> Self {
        Self::with_settings(Settings::default(), monitors, windows)
    }

    pub(super) fn with_settings(
        settings: Settings,
        monitors: usize,
        windows: &[(u64, usize)],
    ) -> Self {
        let mut desktop = FakeDesktop::default();
        for &(id, mon) in windows {
            desktop.windows.insert(wid(id), FakeWindow::app(mon));
        }
        let ids: Vec<WindowId> = windows.iter().map(|&(id, _)| wid(id)).collect();
        let now = Instant::now();
        let (tiler, effects) =
            Tiler::new(settings.clone(), &displays(monitors), &ids, &desktop, now);
        let mut harness = Self {
            tiler,
            desktop,
            now,
            effects: Vec::new(),
            hidden: HiddenWindows::default(),
            settings,
            monitors,
            pipeline: None,
        };
        harness.apply(effects);
        harness
    }

    /// From here on, sends effects through a [`Coalescer`] and records
    /// them in [`Expectations`], as the platform applier does: held
    /// effects reach the desktop only on [`Harness::flush`], and events
    /// that echo our own effects are dropped before the tiler sees them.
    pub(super) fn through_platform(mut self) -> Self {
        self.pipeline = Some(Pipeline::new(self.settings.hiding_support));
        self
    }

    fn apply(&mut self, effects: Vec<Effect>) {
        match self.pipeline.as_mut() {
            None => {
                for effect in &effects {
                    deliver(&mut self.desktop, &mut self.hidden, effect);
                }
            }
            Some(pipeline) => {
                for effect in effects.iter().cloned() {
                    if matches!(effect, Effect::Focus { .. }) {
                        pipeline.expectations.held(&effect);
                    }
                    if let Some(effect) = pipeline.coalescer.admit(effect, self.now) {
                        deliver(&mut self.desktop, &mut self.hidden, &effect);
                        pipeline.expectations.applied(&effect, self.now);
                    }
                }
            }
        }
        self.effects = effects;
    }

    /// Applies everything the coalescer held, as the platform does once
    /// input goes quiet. Does nothing without [`Harness::through_platform`].
    pub(super) fn flush(&mut self) {
        let Some(pipeline) = self.pipeline.as_mut() else {
            return;
        };
        for effect in pipeline.coalescer.take() {
            deliver(&mut self.desktop, &mut self.hidden, &effect);
            pipeline.expectations.applied(&effect, self.now);
        }
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

    /// Reports a window event. Through the platform, an echo of our own
    /// effects is dropped before the tiler sees it.
    pub(super) fn event(&mut self, event: WindowEvent) -> &[Effect] {
        let now = self.now;
        if let Some(pipeline) = self.pipeline.as_mut()
            && pipeline.expectations.echo(&event, now).is_some()
        {
            self.effects.clear();
            return &self.effects;
        }
        self.step(Input::Event(event))
    }

    /// Opens a new visible window on `monitor` and reports it.
    pub(super) fn open(&mut self, id: u64, monitor: usize) -> &[Effect] {
        self.desktop
            .windows
            .insert(wid(id), FakeWindow::app(monitor));
        self.event(WindowEvent::Created { window: wid(id) })
    }

    /// Kills the daemon without a clean shutdown and starts a new one on
    /// the same desktop, the way the platform does: it shows what the
    /// record says the old daemon hid, then tiles whatever exists.
    pub(super) fn kill_and_restart(&mut self) -> &[Effect] {
        let record = std::mem::take(&mut self.hidden);
        // Held effects and expectations die with the old daemon.
        if self.pipeline.is_some() {
            self.pipeline = Some(Pipeline::new(self.settings.hiding_support));
        }
        let desktop = &self.desktop;
        let mut effects = record.restore(|w| desktop.identity(w));
        for effect in &effects {
            self.desktop.apply(effect);
        }
        let mut ids: Vec<WindowId> = self.desktop.windows.keys().copied().collect();
        ids.sort();
        let (tiler, started) = Tiler::new(
            self.settings.clone(),
            &displays(self.monitors),
            &ids,
            &self.desktop,
            self.now,
        );
        self.tiler = tiler;
        for effect in &started {
            deliver(&mut self.desktop, &mut self.hidden, effect);
        }
        effects.extend(started);
        self.effects = effects;
        &self.effects
    }

    pub(super) fn advance(&mut self, by: Duration) {
        self.now += by;
    }

    /// Windows on the active workspace of monitor `mon`, in order.
    pub(super) fn active(&self, mon: usize) -> Vec<u64> {
        self.tiler.monitors()[mon]
            .active()
            .windows()
            .iter()
            .map(|w| w.raw())
            .collect()
    }

    pub(super) fn workspace(&self, mon: usize, ws: usize) -> &Workspace {
        &self.tiler.monitors()[mon].workspaces()[ws]
    }

    pub(super) fn window(&mut self, window: u64) -> &mut FakeWindow {
        self.desktop
            .windows
            .get_mut(&wid(window))
            .expect("window exists")
    }

    pub(super) fn rect_of(&self, window: u64) -> Rect {
        self.desktop.windows[&wid(window)].rect
    }

    /// The window the last step asked to focus, if it asked.
    pub(super) fn focus_request(&self) -> Option<u64> {
        self.effects.iter().rev().find_map(|e| match e {
            Effect::Focus { window, .. } => Some(window.raw()),
            _ => None,
        })
    }

    /// Placements of the last step, as `(window, rect, timing)`.
    pub(super) fn placements(&self) -> Vec<(u64, Rect, Timing)> {
        self.effects
            .iter()
            .filter_map(|e| match *e {
                Effect::Place {
                    window,
                    rect,
                    timing,
                } => Some((window.raw(), rect, timing)),
                _ => None,
            })
            .collect()
    }
}

/// Bare monitor state for tests of the free functions.
pub(super) fn make_monitor(id: u64) -> MonitorState {
    MonitorState {
        id: MonitorId::from_raw(id),
        key: MonitorKey::default(),
        os_work_area: Rect::new(0, 0, MON_W, MON_H),
        work_area: Rect::new(0, 0, MON_W, MON_H),
        workspaces: (0..MAX_WORKSPACES).map(|_| Workspace::new()).collect(),
        active_workspace: 0,
    }
}

pub(super) fn make_monitors(n: usize) -> Vec<MonitorState> {
    (0..n as u64).map(make_monitor).collect()
}

/// Runs the real display change rebuild with unidentified monitors, the
/// first of which is primary.
pub(super) fn simulate_display_change(
    old_monitors: &mut Vec<MonitorState>,
    new_infos: Vec<(u64, Rect)>,
) -> Vec<MonitorState> {
    let infos: Vec<Display> = new_infos
        .into_iter()
        .enumerate()
        .map(|(i, (id, work_area))| Display {
            id: MonitorId::from_raw(id),
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
