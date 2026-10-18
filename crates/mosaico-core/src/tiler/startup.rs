//! What the tiler is started with, and the layout it starts with.

use std::collections::HashSet;
use std::time::Instant;

use crate::action::MAX_WORKSPACES;
use crate::config::{AnimationConfig, BorderConfig, LayoutConfig, WindowRule, WorkspacesConfig};
use crate::{HidingSupport, LayoutKind, WindowId};

use super::{
    Desktop, Display, Effect, MonitorState, Plan, Tiler, bar_states, hidden_how, reconnect,
};

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
    /// Whether windows move to their new place over time, and how.
    pub animations: AnimationConfig,
    /// The ways the platform can hide windows. A `layout.hiding` it does
    /// not have is swapped for one it has.
    pub hiding_support: HidingSupport,
}

impl Tiler {
    /// Starts tiling `windows` across `displays`.
    ///
    /// `displays` and `windows` are what exists right now, in the order
    /// [`Enumerator::monitors`](super::Enumerator::monitors) and
    /// [`Enumerator::windows`](super::Enumerator::windows) list them. The
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
            hidden_how: hidden_how::HiddenHow::default(),
            workspace_mode: settings.workspaces.mode,
            workspace_wrap: settings.workspaces.wrap,
            rules: settings.rules,
            border_config: settings.borders,
            animations: settings.animations,
            fresh: true,
            bordered: HashSet::new(),
            decorated: HashSet::new(),
            mouse_follows_focus: settings.mouse_follows_focus,
            focused_monitor: 0,
            focused_window: None,
            focused_maximized: false,
            filling: bar_states::Filling::default(),
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
        tiler.seed_filling(desktop);
        tiler.retile_all(&mut plan);
        tiler.focus_initial(&mut plan);
        let start = tiler.step_kind(true);
        (tiler, plan.finish(&start))
    }
}
