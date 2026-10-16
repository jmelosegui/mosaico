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
use mosaico_core::tiler::{BarReservation, Display, Enumerator, Input, Settings, Tiler};
use mosaico_core::{Action, HidingSupport, Result, WindowControl, WindowEvent};

use crate::bar::BarState;
use crate::enumerate::WindowsEnumerator;
use crate::window_control::Win32WindowControl;

use applier::Applier;
use desktop::Win32Desktop;
use hidden_file::HiddenFile;

/// Manages tiled windows across all connected monitors.
pub struct TilingManager {
    tiler: Tiler,
    applier: Applier,
    desktop: Win32Desktop,
    control: Win32WindowControl,
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
        let desktop = Win32Desktop::new();
        let control = Win32WindowControl;
        let hiding = HidingSupport::new(|how| control.hiding_events(how));
        let mut applier = Applier::new(HiddenFile::current(), hiding);
        applier.restore_previous(&desktop, &control, Instant::now());

        let displays = WindowsEnumerator.monitors()?;
        let windows = WindowsEnumerator.windows()?;
        let settings = Settings {
            layout: layout_config.clone(),
            workspaces: workspaces_config.clone(),
            borders: border_config,
            rules,
            mouse_follows_focus,
            hiding_support: hiding,
        };
        let now = Instant::now();
        let (tiler, effects) = Tiler::new(settings, &displays, &windows, &desktop, now);
        let mut manager = Self {
            tiler,
            applier,
            desktop,
            control,
        };
        manager
            .applier
            .apply(effects, &manager.desktop, &manager.control, now);
        Ok(manager)
    }

    fn step(&mut self, input: Input) {
        let now = Instant::now();
        let effects = self.tiler.step(input, &self.desktop, now);
        self.applier
            .apply(effects, &self.desktop, &self.control, now);
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

    /// When the focus changes and placements held back, waiting for
    /// input to go quiet, started being held, or `None` when nothing is
    /// held.
    pub fn held_since(&self) -> Option<Instant> {
        self.applier.held_since()
    }

    /// Applies everything held back. The daemon calls this when the
    /// scheduler says it is due.
    pub fn flush_pending(&mut self) {
        self.applier
            .flush(&self.desktop, &self.control, Instant::now());
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
    ///
    /// `new_monitors` are left to right, as [`Enumerator::monitors`]
    /// lists them.
    pub fn handle_display_change(
        &mut self,
        new_monitors: Vec<Display>,
        bar_height: i32,
        bar_monitor_indices: &[usize],
    ) {
        self.step(Input::Displays {
            monitors: new_monitors,
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
        let monitors = WindowsEnumerator.monitors().unwrap_or_default();
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

fn bar(height: i32, monitors: &[usize]) -> BarReservation {
    BarReservation {
        height,
        monitors: monitors.to_vec(),
    }
}
