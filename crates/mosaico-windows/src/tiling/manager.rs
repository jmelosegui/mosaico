//! The tiling manager: feeds the tiler its input and applies the
//! effects it returns.

use std::time::Instant;

use mosaico_core::bar::BarState;
use mosaico_core::config::{BorderConfig, LayoutConfig, WindowRule, WorkspacesConfig};
use mosaico_core::snapshot::DesktopSnapshot;
use mosaico_core::tiler::{Enumerator, HiddenFile, Input, Settings, Tiler};
use mosaico_core::{Action, HidingSupport, Instance, Result, WindowControl, WindowEvent};

use crate::enumerate::WindowsEnumerator;
use crate::window_control::Win32WindowControl;

use super::applier::Applier;
use super::desktop::Win32Desktop;
use super::inspect;

#[path = "manager_inputs.rs"]
mod inputs;

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
        let hidden = Instance::current().map(HiddenFile::for_instance);
        let mut applier = Applier::new(hidden, hiding);
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

    /// Returns the total number of managed windows.
    pub fn window_count(&self) -> usize {
        self.tiler.window_count()
    }

    /// Returns the daemon's internal workspace state, for `debug state`.
    pub fn snapshot(&self) -> DesktopSnapshot {
        inspect::snapshot(&self.tiler, self.desktop.self_elevated())
    }

    /// Returns a snapshot of bar state for each monitor.
    pub fn bar_states(&self, update_text: &str, paused: bool) -> Vec<BarState> {
        self.tiler.bar_states(update_text, paused)
    }
}
