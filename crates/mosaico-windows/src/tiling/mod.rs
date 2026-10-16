//! Drives the core tiler against the Windows desktop.
//!
//! [`mosaico_core::tiler::Tiler`] makes every decision. This module
//! feeds it actions and events, applies the effects it returns through
//! [`applier::Applier`], and drops the events those effects cause.

mod applier;
mod desktop;
mod hidden_file;
mod inspect;

use std::time::Instant;

use mosaico_core::config::{BorderConfig, Config, LayoutConfig, WindowRule, WorkspacesConfig};
use mosaico_core::tiler::{BarReservation, Display, Input, Settings, Tiler};
use mosaico_core::{Action, Result, WindowEvent, WindowId};

use crate::bar::BarState;
use crate::enumerate;
use crate::monitor::{self, MonitorInfo};
use crate::window::Window;

use applier::Applier;
use desktop::Win32Desktop;
use hidden_file::HiddenFile;

/// Manages tiled windows across all connected monitors.
pub struct TilingManager {
    tiler: Tiler,
    applier: Applier,
    desktop: Win32Desktop,
}

impl TilingManager {
    /// Creates a new tiling manager with the given layout, rules, and borders.
    pub fn new(
        layout_config: &LayoutConfig,
        workspaces_config: &WorkspacesConfig,
        rules: Vec<WindowRule>,
        border_config: BorderConfig,
        mouse_follows_focus: bool,
    ) -> Result<Self> {
        // Show what a killed or crashed daemon left hidden before looking
        // at the windows, so they are found and adopted like any other.
        let mut applier = Applier::with_hidden_file(HiddenFile::current());
        applier.restore_previous(Instant::now());

        let displays = displays(monitor::enumerate_monitors()?);
        let windows: Vec<WindowId> = enumerate::enumerate_windows()?
            .iter()
            .map(Window::id)
            .collect();
        let settings = Settings {
            layout: layout_config.clone(),
            workspaces: workspaces_config.clone(),
            borders: border_config,
            rules,
            mouse_follows_focus,
        };
        let desktop = Win32Desktop::new();
        let now = Instant::now();
        let (tiler, effects) = Tiler::new(settings, &displays, &windows, &desktop, now);
        let mut manager = Self {
            tiler,
            applier,
            desktop,
        };
        manager.applier.apply(effects, now);
        Ok(manager)
    }

    fn step(&mut self, input: Input) {
        let now = Instant::now();
        let effects = self.tiler.step(input, &self.desktop, now);
        self.applier.apply(effects, now);
    }

    /// Executes a user-triggered action.
    pub fn handle_action(&mut self, action: &Action) {
        self.step(Input::Action(*action));
    }

    /// Handles a window event, unless it is an echo of our own changes.
    pub fn handle_event(&mut self, event: &WindowEvent) {
        if let WindowEvent::Destroyed { window } = *event {
            self.applier.forget(window);
        }
        if let Some(reason) = self.applier.echo(event, Instant::now()) {
            mosaico_core::log_debug!("{} suppressed ({})", event, reason);
            return;
        }
        self.step(Input::Event(event.clone()));
    }

    /// Whether focus changes or placements are held back, waiting for
    /// input to go quiet.
    pub fn has_pending(&self) -> bool {
        self.applier.has_pending()
    }

    /// Applies everything held back. The daemon calls this once input
    /// has been quiet for a moment.
    pub fn flush_pending(&mut self) {
        self.applier.flush(Instant::now());
    }

    /// Redraws every border to match the current window positions.
    pub fn refresh_border(&mut self) {
        self.step(Input::RefreshBorders);
    }

    /// Repaints every visible window, including those already in place.
    pub fn refresh_visible_windows(&mut self) {
        self.step(Input::RepaintAll);
    }

    /// Shows every window again and restores their frames. Called on
    /// shutdown so nothing is left hidden.
    pub fn restore_all_windows(&mut self) {
        self.step(Input::Shutdown);
    }

    /// Applies a new layout, border and mouse config, then retiles.
    pub fn reload_config(&mut self, config: &Config) {
        self.step(Input::Config(Box::new(config.clone())));
    }

    /// Replaces the window rules and lets go of windows they no longer
    /// manage.
    pub fn reload_rules(&mut self, rules: Vec<WindowRule>) {
        self.step(Input::Rules(rules));
    }

    /// Rebuilds the monitor state after a display configuration change.
    pub fn handle_display_change(
        &mut self,
        new_monitors: Vec<MonitorInfo>,
        bar_height: i32,
        bar_monitor_indices: &[usize],
    ) {
        self.step(Input::Displays {
            monitors: displays(new_monitors),
            bar: bar(bar_height, bar_monitor_indices),
        });
    }

    /// Reserves the bar height at the top of each bar monitor, then
    /// retiles. An empty `bar_monitors` means all monitors.
    pub fn adjust_work_areas_for_bar(&mut self, bar_height: i32, bar_monitors: &[usize]) {
        self.step(Input::Bar(bar(bar_height, bar_monitors)));
    }

    /// Re-reads the OS work areas, then re-reserves the bar and retiles.
    pub fn reset_and_adjust_work_areas(&mut self, bar_height: i32, bar_monitors: &[usize]) {
        let monitors = monitor::enumerate_monitors()
            .map(displays)
            .unwrap_or_default();
        self.step(Input::WorkAreas {
            monitors,
            bar: bar(bar_height, bar_monitors),
        });
    }

    /// Returns the total number of managed windows.
    pub fn window_count(&self) -> usize {
        self.tiler.window_count()
    }

    /// Returns a formatted dump of the daemon's internal workspace state.
    pub fn inspect_state(&self) -> String {
        inspect::describe(&self.tiler, self.desktop.self_elevated())
    }

    /// Returns a snapshot of bar state for each monitor.
    pub fn bar_states(&self, update_text: &str, paused: bool) -> Vec<BarState> {
        self.tiler
            .monitors()
            .iter()
            .enumerate()
            .map(|(i, m)| BarState {
                active_workspace: m.active_workspace(),
                workspace_count: m.workspaces().len(),
                layout_name: m.active().layout_kind().name().into(),
                monocle: m.active().monocle(),
                cpu_usage: 0,
                update_text: update_text.to_string(),
                focused_window: if i == self.tiler.focused_monitor() {
                    self.tiler.focused_window()
                } else {
                    None
                },
                media_text: String::new(),
                paused,
            })
            .collect()
    }
}

/// The monitors as the tiler sees them.
fn displays(monitors: Vec<MonitorInfo>) -> Vec<Display> {
    monitors
        .into_iter()
        .map(|m| Display {
            id: m.id,
            key: m.key,
            work_area: m.work_area,
            primary: m.primary,
        })
        .collect()
}

fn bar(height: i32, monitors: &[usize]) -> BarReservation {
    BarReservation {
        height,
        monitors: monitors.to_vec(),
    }
}
