//! The tiling state machine.
//!
//! [`Tiler`] owns every tiling decision: which windows are managed,
//! where they go, which workspace is on screen and what has focus. It
//! never touches the OS. Each [`Tiler::step`] reads the desktop through
//! [`Desktop`], updates the tiler's state and returns the [`Effect`]s
//! that bring the desktop in line with it. The platform applies them,
//! and records them in an [`Expectations`] so that the events they
//! cause are recognized as ours.

mod adopt;
mod borders;
mod coalesce;
mod corners;
mod decoration_effects;
mod desktop;
mod display;
mod display_rebuild;
mod echo;
mod echo_focus;
mod effect;
mod empty_focus;
mod enumerator;
mod events;
mod events_focus;
mod focus;
mod hidden;
mod input;
mod layout;
mod lifecycle;
mod navigation;
mod navigation_helpers;
mod plan;
mod reconnect;
mod release;
mod snapshot;
mod state;
mod window_effects;
mod work_area;
mod workspace;

use std::collections::HashSet;
use std::time::Instant;

use crate::action::MAX_WORKSPACES;
use crate::config::{
    BorderConfig, HidingBehaviour, LayoutConfig, WindowRule, WorkspaceMode, WorkspacesConfig,
};
use crate::{HidingSupport, LayoutKind, WindowId};

pub use coalesce::Coalescer;
pub use decoration_effects::apply_decoration_effect;
pub use desktop::{Desktop, Display, WindowFacts};
pub use echo::{Expectations, HIDE_FOCUS_ECHO_TTL};
pub use echo_focus::FOCUS_ECHO_TTL;
pub use effect::{BorderStyle, Effect, Timing};
pub use enumerator::{Enumerator, Insets, WindowDetails};
pub use hidden::{HiddenWindows, WindowIdentity};
pub use input::{BarReservation, Input};
pub use release::{MAX_HOLD, QUIET, release_at};
pub use state::MonitorState;
pub use window_effects::apply_window_effect;

use plan::Plan;
use state::SpatialTarget;

/// Everything the tiler is configured with at startup.
#[derive(Debug, Clone, Default)]
pub struct Settings {
    /// Layout algorithm, gaps and hiding.
    pub layout: LayoutConfig,
    /// Workspace mode and per-workspace layouts.
    pub workspaces: WorkspacesConfig,
    /// Border appearance and window corners.
    pub borders: BorderConfig,
    /// Rules deciding which windows are managed.
    pub rules: Vec<WindowRule>,
    /// Whether keyboard-driven focus moves the pointer.
    pub mouse_follows_focus: bool,
    /// The ways the platform can hide windows. A `layout.hiding` it does
    /// not have is swapped for one it has.
    pub hiding_support: HidingSupport,
}

/// Tiled windows across all connected monitors.
pub struct Tiler {
    monitors: Vec<MonitorState>,
    /// Layout of each workspace on a monitor seen for the first time,
    /// from `[workspaces.layouts]` falling back to `[layout] default`.
    workspace_layouts: Vec<LayoutKind>,
    /// Monitors that disconnected and the windows displaced from them.
    reconnect: reconnect::ReconnectMemory,
    /// Layout settings, with `hiding` the way windows are really hidden:
    /// the configured one, or what the platform has instead.
    layout: LayoutConfig,
    /// The hiding strategy the configuration asks for.
    configured_hiding: HidingBehaviour,
    /// The ways the platform can hide windows, to choose from again when
    /// the configured strategy changes.
    hiding_support: HidingSupport,
    /// Whether workspace switches affect one monitor or all monitors.
    workspace_mode: WorkspaceMode,
    rules: Vec<WindowRule>,
    border_config: BorderConfig,
    /// Windows that currently have a border, so the ones that leave the
    /// screen can have theirs destroyed.
    bordered: HashSet<WindowId>,
    /// Windows given the configured corners, so that only those get the
    /// system's corners back. A minimized window leaves the layout but
    /// stays here; one dropped from it for no longer being shown does
    /// not.
    decorated: HashSet<WindowId>,
    mouse_follows_focus: bool,
    focused_monitor: usize,
    focused_window: Option<WindowId>,
    /// Whether the focused window was maximized when last looked at.
    ///
    /// Lets `LocationChanged`, which fires on every animation frame,
    /// react only when the window enters or leaves maximized state.
    focused_maximized: bool,
    /// Pins `focused_monitor` while its active workspace is empty. See
    /// [`empty_focus`].
    empty_focus_claim: Option<empty_focus::EmptyFocusClaim>,
    /// Windows the rules already rejected, so the deferred events that
    /// keep arriving for them are not judged again. Cleared when the
    /// rules change; entries are removed when the window is destroyed.
    adopt_rejected: HashSet<WindowId>,
}

impl Tiler {
    /// Starts tiling `windows` across `displays`.
    ///
    /// `displays` and `windows` are what exists right now, in the order
    /// [`Enumerator::monitors`] and [`Enumerator::windows`] list them. The
    /// tiler keeps the windows the rules accept, each on the monitor it is
    /// on, and returns the effects that lay them out.
    pub fn new(
        settings: Settings,
        displays: &[Display],
        windows: &[WindowId],
        desktop: &dyn Desktop,
        now: Instant,
    ) -> (Self, Vec<Effect>) {
        let workspace_layouts: Vec<LayoutKind> = (1..=MAX_WORKSPACES)
            .map(|n| {
                settings
                    .workspaces
                    .layouts
                    .get(&n)
                    .copied()
                    .unwrap_or(settings.layout.default)
            })
            .collect();
        let monitors = displays
            .iter()
            .map(|d| MonitorState::new(d, &workspace_layouts))
            .collect();
        let mut layout = settings.layout;
        let configured_hiding = layout.hiding;
        layout.hiding = settings.hiding_support.choose(configured_hiding);

        let mut tiler = Self {
            monitors,
            workspace_layouts,
            reconnect: reconnect::ReconnectMemory::default(),
            layout,
            configured_hiding,
            hiding_support: settings.hiding_support,
            workspace_mode: settings.workspaces.mode,
            rules: settings.rules,
            border_config: settings.borders,
            bordered: HashSet::new(),
            decorated: HashSet::new(),
            mouse_follows_focus: settings.mouse_follows_focus,
            focused_monitor: 0,
            focused_window: None,
            focused_maximized: false,
            empty_focus_claim: None,
            adopt_rejected: HashSet::new(),
        };

        let mut plan = Plan::new(desktop, now);
        for &window in windows {
            if tiler.is_tileable(&plan, window)
                && let Some(idx) = tiler.monitor_index_for(&plan, window)
            {
                tiler.monitors[idx].active_ws_mut().add(window);
                tiler.decorate(&mut plan, window);
            }
        }
        tiler.retile_all(&mut plan);
        tiler.focus_initial(&mut plan);
        (tiler, plan.finish())
    }

    /// Handles one input and returns the effects to apply, in order.
    pub fn step(&mut self, input: Input, desktop: &dyn Desktop, now: Instant) -> Vec<Effect> {
        let mut plan = Plan::new(desktop, now);
        match input {
            Input::Action(action) => self.handle_action(&mut plan, &action),
            Input::Event(event) => self.handle_event(&mut plan, &event),
            Input::Displays { monitors, bar } => {
                self.handle_display_change(&mut plan, monitors, &bar)
            }
            Input::WorkAreas { monitors, bar } => self.reset_work_areas(&mut plan, &monitors, &bar),
            Input::Bar(bar) => self.reserve_bar(&mut plan, &bar),
            Input::Config(config) => self.reload_config(&mut plan, &config),
            Input::Rules(rules) => self.reload_rules(&mut plan, rules),
            Input::RefreshBorders => self.update_border(&mut plan),
            Input::RepaintAll => self.repaint_visible(&mut plan),
            Input::Shutdown => self.restore_all_windows(&mut plan),
        }
        plan.finish()
    }

    /// Every monitor, in the order of the displays handed to
    /// [`Tiler::new`] or in the latest [`Input::Displays`]: left to right
    /// when they come from [`Enumerator::monitors`].
    pub fn monitors(&self) -> &[MonitorState] {
        &self.monitors
    }

    /// Index of the monitor the user is working on.
    pub fn focused_monitor(&self) -> usize {
        self.focused_monitor
    }

    /// The window that has focus, as far as the tiler is concerned.
    pub fn focused_window(&self) -> Option<WindowId> {
        self.focused_window
    }

    /// Total number of managed windows.
    pub fn window_count(&self) -> usize {
        self.monitors
            .iter()
            .flat_map(|m| &m.workspaces)
            .map(|ws| ws.len())
            .sum()
    }
}

#[cfg(test)]
mod tests;
