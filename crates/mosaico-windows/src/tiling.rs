use mosaico_core::window::Window as WindowTrait;
use mosaico_core::{Action, BspLayout, Rect, WindowEvent, WindowResult, Workspace};

use crate::enumerate;
use crate::monitor;
use crate::window::Window;

/// Per-monitor state: workspace and cached work area.
struct MonitorState {
    id: usize,
    work_area: Rect,
    workspace: Workspace,
}

/// Manages tiled windows across all connected monitors.
///
/// Each monitor gets its own workspace. Focus and swap actions operate
/// within the focused monitor; additional actions move focus or windows
/// between monitors.
pub struct TilingManager {
    monitors: Vec<MonitorState>,
    layout: BspLayout,
    focused_monitor: usize,
    focused_window: Option<usize>,
}

impl TilingManager {
    /// Creates a new tiling manager, discovers monitors, and populates
    /// workspaces with existing windows.
    pub fn new(layout: BspLayout) -> WindowResult<Self> {
        let monitors: Vec<MonitorState> = monitor::enumerate_monitors()?
            .into_iter()
            .map(|info| MonitorState {
                id: info.id,
                work_area: info.work_area,
                workspace: Workspace::new(),
            })
            .collect();

        let mut manager = Self {
            monitors,
            layout,
            focused_monitor: 0,
            focused_window: None,
        };

        for win in enumerate::enumerate_windows()? {
            let hwnd = win.hwnd().0 as usize;
            if let Some(idx) = manager.monitor_index_for(hwnd) {
                manager.monitors[idx].workspace.add(hwnd);
            }
        }

        manager.retile_all();
        Ok(manager)
    }

    /// Handles a window event and re-tiles the affected monitor.
    pub fn handle_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::Created { hwnd } | WindowEvent::Restored { hwnd } => {
                let window = Window::from_raw(*hwnd);
                if !window.is_visible() {
                    return;
                }
                if let Some(idx) = self.monitor_index_for(*hwnd)
                    && self.monitors[idx].workspace.add(*hwnd)
                {
                    self.apply_layout_on(idx);
                }
            }
            WindowEvent::Destroyed { hwnd } | WindowEvent::Minimized { hwnd } => {
                if let Some(idx) = self.owning_monitor(*hwnd)
                    && self.monitors[idx].workspace.remove(*hwnd)
                {
                    self.apply_layout_on(idx);
                }
            }
            WindowEvent::Focused { hwnd } => {
                self.focused_window = Some(*hwnd);
                if let Some(idx) = self.owning_monitor(*hwnd) {
                    self.focused_monitor = idx;
                }
            }
            _ => {}
        }
    }

    /// Executes a user-triggered action.
    pub fn handle_action(&mut self, action: &Action) {
        match action {
            Action::FocusNext => self.focus_adjacent(1),
            Action::FocusPrev => self.focus_adjacent(-1),
            Action::SwapNext => self.swap_adjacent(1),
            Action::SwapPrev => self.swap_adjacent(-1),
            Action::Retile => self.retile_all(),
            Action::FocusMonitorNext => self.focus_monitor(1),
            Action::FocusMonitorPrev => self.focus_monitor(-1),
            Action::MoveToMonitorNext => self.move_to_monitor(1),
            Action::MoveToMonitorPrev => self.move_to_monitor(-1),
        }
    }

    /// Returns the total number of managed windows across all monitors.
    pub fn window_count(&self) -> usize {
        self.monitors.iter().map(|m| m.workspace.len()).sum()
    }

    /// Moves focus to an adjacent window on the focused monitor.
    fn focus_adjacent(&mut self, direction: i32) {
        let ws = &self.monitors[self.focused_monitor].workspace;
        let Some(idx) = self.focused_window.and_then(|h| ws.index_of(h)) else {
            return;
        };
        let len = ws.len() as i32;
        let next = ((idx as i32 + direction).rem_euclid(len)) as usize;
        if let Some(&hwnd) = ws.handles().get(next) {
            self.focused_window = Some(hwnd);
            Window::from_raw(hwnd).set_foreground();
        }
    }

    /// Swaps the focused window with an adjacent one and re-tiles.
    fn swap_adjacent(&mut self, direction: i32) {
        let ws = &self.monitors[self.focused_monitor].workspace;
        let Some(idx) = self.focused_window.and_then(|h| ws.index_of(h)) else {
            return;
        };
        let len = ws.len() as i32;
        let other = ((idx as i32 + direction).rem_euclid(len)) as usize;
        self.monitors[self.focused_monitor]
            .workspace
            .swap(idx, other);
        self.apply_layout_on(self.focused_monitor);
    }

    /// Switches focus to the first window on the next/previous monitor.
    fn focus_monitor(&mut self, direction: i32) {
        if self.monitors.len() <= 1 {
            return;
        }
        let len = self.monitors.len() as i32;
        let next = ((self.focused_monitor as i32 + direction).rem_euclid(len)) as usize;
        self.focused_monitor = next;
        if let Some(&hwnd) = self.monitors[next].workspace.handles().first() {
            self.focused_window = Some(hwnd);
            Window::from_raw(hwnd).set_foreground();
        }
    }

    /// Moves the focused window to the next/previous monitor and re-tiles both.
    fn move_to_monitor(&mut self, direction: i32) {
        let Some(hwnd) = self.focused_window else {
            return;
        };
        if self.monitors.len() <= 1 {
            return;
        }
        let source = self.focused_monitor;
        let len = self.monitors.len() as i32;
        let target = ((source as i32 + direction).rem_euclid(len)) as usize;
        if source == target {
            return;
        }
        self.monitors[source].workspace.remove(hwnd);
        self.monitors[target].workspace.add(hwnd);
        self.apply_layout_on(source);
        self.apply_layout_on(target);
        self.focused_monitor = target;
    }

    /// Re-tiles all monitors.
    fn retile_all(&self) {
        for i in 0..self.monitors.len() {
            self.apply_layout_on(i);
        }
    }

    /// Applies the layout to a single monitor's workspace.
    fn apply_layout_on(&self, monitor_idx: usize) {
        let state = &self.monitors[monitor_idx];
        let positions = state
            .workspace
            .compute_layout(&self.layout, &state.work_area);
        for (hwnd, rect) in &positions {
            let window = Window::from_raw(*hwnd);
            if let Err(e) = window.set_rect(rect) {
                eprintln!("Failed to position window 0x{hwnd:X}: {e}");
            }
        }
    }

    /// Finds which monitor a window belongs to (via `MonitorFromWindow`).
    fn monitor_index_for(&self, hwnd: usize) -> Option<usize> {
        let mid = monitor::monitor_id_for_window(hwnd);
        self.monitors.iter().position(|m| m.id == mid)
    }

    /// Finds which monitor currently has the window in its workspace.
    fn owning_monitor(&self, hwnd: usize) -> Option<usize> {
        self.monitors
            .iter()
            .position(|m| m.workspace.contains(hwnd))
    }
}
