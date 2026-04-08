mod display;
mod event_handler;
mod focus;
mod helpers;
mod layout;
mod lifecycle;
mod navigation;
mod navigation_helpers;
mod workspace;

use std::collections::HashSet;
use std::time::Instant;

use mosaico_core::action::MAX_WORKSPACES;
use mosaico_core::config::{BorderConfig, HidingBehaviour, LayoutConfig, WindowRule};
use mosaico_core::{Action, Rect, WindowResult, Workspace};

use crate::bar::BarState;
use crate::border::Border;
use crate::enumerate;
use crate::frame;
use crate::monitor;
use crate::window::Window;

/// Per-monitor state: multiple workspaces and work area.
pub(super) struct MonitorState {
    pub(super) id: usize,
    pub(super) work_area: Rect,
    pub(super) workspaces: Vec<Workspace>,
    pub(super) active_workspace: usize,
}

impl MonitorState {
    fn active_ws(&self) -> &Workspace {
        &self.workspaces[self.active_workspace]
    }

    fn active_ws_mut(&mut self) -> &mut Workspace {
        &mut self.workspaces[self.active_workspace]
    }
}

/// Result of resolving a directional (H/L) spatial action.
///
/// Used by both focus and move operations to share the spatial
/// neighbor lookup and monitor-overflow logic.
enum SpatialTarget {
    /// A neighbor window on the same monitor.
    Neighbor(usize),
    /// The adjacent monitor when no same-monitor neighbor exists.
    AdjacentMonitor(usize),
}

/// Manages tiled windows across all connected monitors.
pub struct TilingManager {
    monitors: Vec<MonitorState>,
    layout_gap: i32,
    layout_ratio: f64,
    rules: Vec<WindowRule>,
    border: Option<Border>,
    border_config: BorderConfig,
    focused_monitor: usize,
    focused_window: Option<usize>,
    /// Suppresses `Moved` event handling during programmatic layout.
    applying_layout: bool,
    /// Tracks whether the focused window is currently maximized.
    ///
    /// Used to detect maximize/restore transitions from
    /// `LocationChanged` events without calling `update_border()`
    /// on every animation frame.
    focused_maximized: bool,
    /// Whether keyboard-driven focus should move the cursor.
    mouse_follows_focus: bool,
    /// Suppresses cursor moves after mouse-driven focus.
    focus_from_mouse: bool,
    /// How windows are hidden during workspace switches.
    hiding: HidingBehaviour,
    /// Windows hidden programmatically by workspace switching.
    ///
    /// Events for these hwnds are ignored until they are shown again.
    /// This prevents `EVENT_OBJECT_HIDE` from removing windows that
    /// were just hidden by a workspace switch.
    hidden_by_switch: HashSet<usize>,
    /// Cooldown timestamp suppressing focus-triggered workspace switches.
    ///
    /// Win32 events fired during a workspace switch (e.g. by
    /// `set_foreground()` or `SetWindowLongW` inside `set_rect`)
    /// are queued asynchronously and may arrive after the switch
    /// completes.  Checking a boolean guard is insufficient because
    /// the flag is cleared before these deferred events are processed.
    ///
    /// Instead we record the `Instant` when the switch finishes and
    /// suppress focus-triggered switches until a short cooldown
    /// elapses, preventing infinite workspace-switching loops.
    ws_switch_cooldown: Option<Instant>,
    /// Whether the current process (mosaico) is running elevated.
    ///
    /// When `false`, elevated windows are skipped in `is_tileable`
    /// because `SetWindowPos` silently fails across the UIPI boundary.
    self_elevated: bool,
    /// Windows that `try_adopt` already rejected via `is_tileable`.
    ///
    /// Prevents re-evaluating expensive Win32 queries (visibility,
    /// elevation, class, title) on every deferred event for windows
    /// that will never be managed (e.g. elevated Visual Studio).
    /// Cleared on rule reload; entries removed on `Destroyed`.
    adopt_rejected: HashSet<usize>,
}

impl TilingManager {
    /// Creates a new tiling manager with the given layout, rules, and borders.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        layout_config: &LayoutConfig,
        rules: Vec<WindowRule>,
        border_config: BorderConfig,
        mouse_follows_focus: bool,
    ) -> WindowResult<Self> {
        let monitors: Vec<MonitorState> = monitor::enumerate_monitors()?
            .into_iter()
            .map(|info| MonitorState {
                id: info.id,
                work_area: info.work_area,
                workspaces: (0..MAX_WORKSPACES)
                    .map(|i| {
                        let ws_num = i + 1;
                        let kind = layout_config
                            .workspaces
                            .get(&ws_num)
                            .copied()
                            .unwrap_or(layout_config.default);
                        Workspace::with_layout(kind)
                    })
                    .collect(),
                active_workspace: 0,
            })
            .collect();

        let border = Border::new().ok();
        let self_elevated = crate::process::is_current_process_elevated();

        let mut manager = Self {
            monitors,
            layout_gap: layout_config.gap,
            layout_ratio: layout_config.ratio,
            rules,
            border,
            border_config,
            focused_monitor: 0,
            focused_window: None,
            focused_maximized: false,
            mouse_follows_focus,
            focus_from_mouse: false,
            applying_layout: false,
            hiding: layout_config.hiding,
            hidden_by_switch: HashSet::new(),
            ws_switch_cooldown: None,
            self_elevated,
            adopt_rejected: HashSet::new(),
        };

        for win in enumerate::enumerate_windows()? {
            let hwnd = win.hwnd().0 as usize;
            if manager.is_tileable(hwnd)
                && let Some(idx) = manager.monitor_index_for(hwnd)
            {
                manager.monitors[idx].active_ws_mut().add(hwnd);
                frame::set_corner_preference(win.hwnd(), manager.border_config.corner_style);
            }
        }

        manager.retile_all();
        manager.focus_initial();
        Ok(manager)
    }

    /// Executes a user-triggered action.
    pub fn handle_action(&mut self, action: &Action) {
        match action {
            Action::Focus(dir) => {
                self.focus_from_mouse = false;
                self.focus_direction(*dir);
            }
            Action::Move(dir) => self.move_direction(*dir),
            Action::Retile => self.retile_all(),
            Action::ToggleMonocle => self.toggle_monocle(),
            Action::CloseFocused => self.close_focused(),
            Action::MinimizeFocused => self.minimize_focused(),
            Action::CycleLayout => self.cycle_layout(),
            Action::GoToWorkspace(n) => self.goto_workspace(*n),
            Action::SendToWorkspace(n) => self.send_to_workspace(*n),
            // TogglePause is handled by the daemon before reaching here.
            Action::TogglePause => {}
        }
    }

    /// Returns the total number of managed windows.
    pub fn window_count(&self) -> usize {
        self.monitors
            .iter()
            .flat_map(|m| &m.workspaces)
            .map(|ws| ws.len())
            .sum()
    }

    /// Returns a formatted dump of the daemon's internal workspace state.
    pub fn inspect_state(&self) -> String {
        use std::fmt::Write;

        let mut out = String::new();

        for (mi, mon) in self.monitors.iter().enumerate() {
            let _ = writeln!(
                out,
                "Monitor {} (id={}, work_area={}x{}+{}+{}, active_ws={}):",
                mi,
                mon.id,
                mon.work_area.width,
                mon.work_area.height,
                mon.work_area.x,
                mon.work_area.y,
                mon.active_workspace + 1,
            );

            for (wi, ws) in mon.workspaces.iter().enumerate() {
                if ws.is_empty() {
                    continue;
                }
                let _ = writeln!(
                    out,
                    "  Workspace {} ({} window{}):",
                    wi + 1,
                    ws.len(),
                    if ws.len() == 1 { "" } else { "s" },
                );
                for &hwnd in ws.handles() {
                    let win = crate::window::Window::from_raw(hwnd);
                    let title = mosaico_core::Window::title(&win).unwrap_or_default();
                    let class = mosaico_core::Window::class(&win).unwrap_or_default();
                    let visible = mosaico_core::Window::is_visible(&win);
                    let minimized = win.is_minimized();
                    let cloaked = win.is_cloaked();
                    let elevated = win.is_elevated();
                    let rect = mosaico_core::Window::rect(&win).ok();
                    let rect_str = match rect {
                        Some(r) => format!("{}x{}+{}+{}", r.width, r.height, r.x, r.y),
                        None => "N/A".to_string(),
                    };
                    let _ = writeln!(
                        out,
                        "    0x{:X}  visible={}  minimized={}  cloaked={}  elevated={}  rect={}  class={:?}  title={:?}",
                        hwnd, visible, minimized, cloaked, elevated, rect_str, class, title,
                    );
                }
            }
        }

        let _ = writeln!(out, "Self elevated: {}", self.self_elevated);
        let _ = writeln!(out, "Focused monitor: {}", self.focused_monitor);
        let _ = match self.focused_window {
            Some(h) => writeln!(out, "Focused window: 0x{:X}", h),
            None => writeln!(out, "Focused window: None"),
        };

        out
    }

    /// Returns a snapshot of bar state for each monitor.
    pub fn bar_states(&self, update_text: &str, paused: bool) -> Vec<BarState> {
        self.monitors
            .iter()
            .enumerate()
            .map(|(i, m)| BarState {
                active_workspace: m.active_workspace,
                workspace_count: m.workspaces.len(),
                layout_name: m.active_ws().layout_kind().name().into(),
                monocle: m.active_ws().monocle(),
                cpu_usage: 0,
                update_text: update_text.to_string(),
                focused_hwnd: if i == self.focused_monitor {
                    self.focused_window
                } else {
                    None
                },
                media_text: String::new(),
                paused,
            })
            .collect()
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
