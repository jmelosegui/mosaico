//! Tiler tests against a fake desktop.
//!
//! [`Harness`] owns a tiler and a [`FakeDesktop`] and applies every
//! effect the tiler returns to the fake straight away, so a test can
//! drive several steps and assert on both the effects and the
//! resulting state. Nothing is held back and no echo is dropped: tests
//! of that run the tiling manager on a `FakePlatform`.

use std::time::{Duration, Instant};

use crate::action::MAX_WORKSPACES;
use crate::config::HidingBehaviour;
use crate::id::wid;
use crate::{Action, MonitorId, MonitorKey, Rect, WindowEvent, WindowId, Workspace};

use super::*;

use fake_desktop::{FakeDesktop, FakeWindow};

#[path = "tests/border_tests.rs"]
mod border_tests;
#[path = "tests/corner_tests.rs"]
mod corner_tests;
#[path = "tests/corner_tracking_tests.rs"]
mod corner_tracking_tests;
#[path = "tests/decoration_effects_tests.rs"]
mod decoration_effects_tests;
#[path = "tests/display_focus_tests.rs"]
mod display_focus_tests;
#[path = "tests/display_tests.rs"]
mod display_tests;
#[path = "tests/empty_focus_tests.rs"]
mod empty_focus_tests;
#[path = "tests/event_tests.rs"]
mod event_tests;
#[path = "tests/fake_desktop.rs"]
mod fake_desktop;
#[path = "tests/fake_platform.rs"]
mod fake_platform;
#[path = "tests/fallback_tests.rs"]
mod fallback_tests;
#[path = "tests/focus_tests.rs"]
mod focus_tests;
#[path = "tests/global_lineup_tests.rs"]
mod global_lineup_tests;
#[path = "tests/hidden_how_tests.rs"]
mod hidden_how_tests;
#[path = "tests/hiding_change_record_tests.rs"]
mod hiding_change_record_tests;
#[path = "tests/hiding_change_tests.rs"]
mod hiding_change_tests;
#[path = "tests/layout_tests.rs"]
mod layout_tests;
#[path = "tests/manager_config_tests.rs"]
mod manager_config_tests;
#[path = "tests/manager_focus_tests.rs"]
mod manager_focus_tests;
#[path = "tests/manager_record_tests.rs"]
mod manager_record_tests;
#[path = "tests/manager_support.rs"]
mod manager_support;
#[path = "tests/manager_tests.rs"]
mod manager_tests;
#[path = "tests/mode_reload_settings_tests.rs"]
mod mode_reload_settings_tests;
#[path = "tests/mode_reload_tests.rs"]
mod mode_reload_tests;
#[path = "tests/monitor_return_tests.rs"]
mod monitor_return_tests;
#[path = "tests/monocle_display_tests.rs"]
mod monocle_display_tests;
#[path = "tests/monocle_join_tests.rs"]
mod monocle_join_tests;
#[path = "tests/monocle_out_of_view_tests.rs"]
mod monocle_out_of_view_tests;
#[path = "tests/monocle_return_tests.rs"]
mod monocle_return_tests;
#[path = "tests/monocle_stale_tests.rs"]
mod monocle_stale_tests;
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
#[path = "tests/stacking_tests.rs"]
mod stacking_tests;
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

    /// Reports a window event. A window reported focused is in front,
    /// whoever focused it.
    pub(super) fn event(&mut self, event: WindowEvent) -> &[Effect] {
        if let WindowEvent::Focused { window } = event {
            self.desktop.raise(window);
        }
        self.step(Input::Event(event))
    }

    /// Opens a new visible window on `monitor` and reports it.
    pub(super) fn open(&mut self, id: u64, monitor: usize) -> &[Effect] {
        self.desktop
            .windows
            .insert(wid(id), FakeWindow::app(monitor));
        self.desktop.raise(wid(id));
        self.event(WindowEvent::Created { window: wid(id) })
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
