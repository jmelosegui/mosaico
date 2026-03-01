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

use mosaico_core::action::MAX_WORKSPACES;
use mosaico_core::config::{BorderConfig, HidingBehaviour, WindowRule};
use mosaico_core::{Action, BspLayout, Rect, WindowResult, Workspace};

use crate::bar::BarState;
use crate::border::Border;
use crate::enumerate;
use crate::frame;
use crate::monitor;
use crate::window::Window;

/// Per-monitor state: multiple workspaces, work area, and monocle flag.
pub(super) struct MonitorState {
    pub(super) id: usize,
    pub(super) work_area: Rect,
    pub(super) workspaces: Vec<Workspace>,
    pub(super) active_workspace: usize,
    pub(super) monocle: bool,
    /// The window shown fullscreen in monocle mode.
    pub(super) monocle_window: Option<usize>,
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
    layout: BspLayout,
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
}

impl TilingManager {
    /// Creates a new tiling manager with the given layout, rules, and borders.
    pub fn new(
        layout: BspLayout,
        rules: Vec<WindowRule>,
        border_config: BorderConfig,
        hiding: HidingBehaviour,
        mouse_follows_focus: bool,
    ) -> WindowResult<Self> {
        let monitors: Vec<MonitorState> = monitor::enumerate_monitors()?
            .into_iter()
            .map(|info| MonitorState {
                id: info.id,
                work_area: info.work_area,
                workspaces: (0..MAX_WORKSPACES).map(|_| Workspace::new()).collect(),
                active_workspace: 0,
                monocle: false,
                monocle_window: None,
            })
            .collect();

        let border = Border::new().ok();

        let mut manager = Self {
            monitors,
            layout,
            rules,
            border,
            border_config,
            focused_monitor: 0,
            focused_window: None,
            focused_maximized: false,
            mouse_follows_focus,
            focus_from_mouse: false,
            applying_layout: false,
            hiding,
            hidden_by_switch: HashSet::new(),
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
            Action::GoToWorkspace(n) => self.goto_workspace(*n),
            Action::SendToWorkspace(n) => self.send_to_workspace(*n),
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

    /// Returns a snapshot of bar state for each monitor.
    pub fn bar_states(&self, update_text: &str) -> Vec<BarState> {
        self.monitors
            .iter()
            .enumerate()
            .map(|(i, m)| BarState {
                active_workspace: m.active_workspace,
                workspace_count: m.workspaces.len(),
                layout_name: "BSP".into(),
                monocle: m.monocle,
                cpu_usage: 0,
                update_text: update_text.to_string(),
                focused_hwnd: if i == self.focused_monitor {
                    self.focused_window
                } else {
                    None
                },
                media_text: String::new(),
            })
            .collect()
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
