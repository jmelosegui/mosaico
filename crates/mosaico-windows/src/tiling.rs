use mosaico_core::config::{BorderConfig, WindowRule};
use mosaico_core::window::Window as WindowTrait;
use mosaico_core::{Action, BspLayout, Rect, WindowEvent, WindowResult, Workspace};

use crate::border::{Border, Color};
use crate::enumerate;
use crate::monitor;
use crate::window::Window;

/// Per-monitor state: workspace, work area, and monocle flag.
struct MonitorState {
    id: usize,
    work_area: Rect,
    workspace: Workspace,
    monocle: bool,
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
}

impl TilingManager {
    /// Creates a new tiling manager with the given layout, rules, and borders.
    pub fn new(
        layout: BspLayout,
        rules: Vec<WindowRule>,
        border_config: BorderConfig,
    ) -> WindowResult<Self> {
        let monitors: Vec<MonitorState> = monitor::enumerate_monitors()?
            .into_iter()
            .map(|info| MonitorState {
                id: info.id,
                work_area: info.work_area,
                workspace: Workspace::new(),
                monocle: false,
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
            applying_layout: false,
        };

        for win in enumerate::enumerate_windows()? {
            let hwnd = win.hwnd().0 as usize;
            if manager.is_tileable(hwnd)
                && let Some(idx) = manager.monitor_index_for(hwnd)
            {
                manager.monitors[idx].workspace.add(hwnd);
            }
        }

        manager.retile_all();
        manager.focus_initial();
        Ok(manager)
    }

    /// Handles a window event and re-tiles the affected monitor.
    pub fn handle_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::Created { hwnd } | WindowEvent::Restored { hwnd } => {
                if !self.is_tileable(*hwnd) {
                    return;
                }
                // Place new windows on the focused monitor so they appear
                // where the user is working, not wherever the OS spawns them.
                let idx = self.focused_monitor;
                if self.monitors.get(idx).is_some() && self.monitors[idx].workspace.add(*hwnd) {
                    let title = Window::from_raw(*hwnd).title().unwrap_or_default();
                    log(format_args!(
                        "+add 0x{:X} \"{}\" to mon {} (now {})",
                        hwnd,
                        title,
                        idx,
                        self.monitors[idx].workspace.len()
                    ));
                    self.apply_layout_on(idx);
                }
            }
            WindowEvent::Destroyed { hwnd } | WindowEvent::Minimized { hwnd } => {
                let owner = self.owning_monitor(*hwnd);
                if let Some(idx) = owner
                    && self.monitors[idx].workspace.remove(*hwnd)
                {
                    log(format_args!(
                        "-del 0x{:X} from mon {} (now {})",
                        hwnd,
                        idx,
                        self.monitors[idx].workspace.len()
                    ));
                    self.apply_layout_on(idx);
                } else {
                    log(format_args!(
                        "-del 0x{:X} MISSED (owning_monitor={:?})",
                        hwnd, owner
                    ));
                }
            }
            WindowEvent::Moved { hwnd } => {
                if !self.applying_layout {
                    self.reassign_monitor(*hwnd);
                }
            }
            WindowEvent::Focused { hwnd } => {
                self.focused_window = Some(*hwnd);
                if let Some(idx) = self.owning_monitor(*hwnd) {
                    self.focused_monitor = idx;
                    self.update_border();
                } else {
                    self.hide_border();
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
            Action::ToggleMonocle => self.toggle_monocle(),
            Action::CloseFocused => self.close_focused(),
        }
    }

    /// Returns the total number of managed windows.
    pub fn window_count(&self) -> usize {
        self.monitors.iter().map(|m| m.workspace.len()).sum()
    }

    fn is_tileable(&self, hwnd: usize) -> bool {
        let window = Window::from_raw(hwnd);
        if !window.is_visible() || !window.is_app_window() {
            return false;
        }
        let class = window.class().unwrap_or_default();
        let title = window.title().unwrap_or_default();
        mosaico_core::config::should_manage(&class, &title, &self.rules)
    }

    /// Focuses the first window on the primary monitor at startup.
    fn focus_initial(&mut self) {
        if let Some(mon) = self.monitors.first()
            && let Some(&hwnd) = mon.workspace.handles().first()
        {
            self.focused_window = Some(hwnd);
            Window::from_raw(hwnd).set_foreground();
            self.update_border();
        }
    }

    fn close_focused(&mut self) {
        let Some(hwnd) = self.focused_window else {
            return;
        };
        // Send WM_CLOSE so the window can handle its own shutdown gracefully.
        use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_CLOSE};
        unsafe {
            let _ = PostMessageW(
                Some(windows::Win32::Foundation::HWND(hwnd as *mut _)),
                WM_CLOSE,
                windows::Win32::Foundation::WPARAM(0),
                windows::Win32::Foundation::LPARAM(0),
            );
        }
    }

    fn focus_adjacent(&mut self, direction: i32) {
        if self.monitors.is_empty() {
            return;
        }
        let ws = &self.monitors[self.focused_monitor].workspace;
        let idx = match self.focused_window.and_then(|h| ws.index_of(h)) {
            Some(i) => i,
            None => {
                // Focused window is not in the workspace — recover by
                // focusing the first available window.
                if let Some(&hwnd) = ws.handles().first() {
                    self.focused_window = Some(hwnd);
                    Window::from_raw(hwnd).set_foreground();
                    self.update_border();
                }
                return;
            }
        };
        let len = ws.len() as i32;
        let next = ((idx as i32 + direction).rem_euclid(len)) as usize;
        if let Some(&hwnd) = ws.handles().get(next) {
            self.focused_window = Some(hwnd);
            Window::from_raw(hwnd).set_foreground();
            self.update_border();
        }
    }

    fn swap_adjacent(&mut self, direction: i32) {
        if self.monitors.is_empty() {
            return;
        }
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

    fn focus_monitor(&mut self, direction: i32) {
        if self.monitors.is_empty() {
            return;
        }

        let Some(focused_hwnd) = self.focused_window else {
            // No focused window — recover by focusing the first window
            // on the current monitor.
            if let Some(&hwnd) = self.monitors[self.focused_monitor]
                .workspace
                .handles()
                .first()
            {
                self.focused_window = Some(hwnd);
                Window::from_raw(hwnd).set_foreground();
                self.update_border();
            }
            return;
        };

        // Compute layout to get spatial positions of all windows.
        let state = &self.monitors[self.focused_monitor];
        let positions = state
            .workspace
            .compute_layout(&self.layout, &state.work_area);

        let Some(focused_rect) = positions
            .iter()
            .find(|(h, _)| *h == focused_hwnd)
            .map(|(_, r)| *r)
        else {
            // Focused window not found in workspace — recover.
            if let Some(&hwnd) = state.workspace.handles().first() {
                self.focused_window = Some(hwnd);
                Window::from_raw(hwnd).set_foreground();
                self.update_border();
            }
            return;
        };

        // Find the best spatial neighbor in the requested direction.
        if let Some(neighbor) = self.find_spatial_neighbor(&positions, &focused_rect, direction) {
            self.focused_window = Some(neighbor);
            Window::from_raw(neighbor).set_foreground();
            self.update_border();
            return;
        }

        // No neighbor on this monitor — overflow to the adjacent monitor.
        if self.monitors.len() <= 1 {
            return;
        }
        let len = self.monitors.len() as i32;
        let next = ((self.focused_monitor as i32 + direction).rem_euclid(len)) as usize;
        self.focused_monitor = next;
        let handles = self.monitors[next].workspace.handles();
        let target = if direction > 0 {
            handles.first()
        } else {
            handles.last()
        };
        if let Some(&hwnd) = target {
            self.focused_window = Some(hwnd);
            Window::from_raw(hwnd).set_foreground();
            self.update_border();
        }
    }

    /// Finds the closest window to the left (direction = -1) or right
    /// (direction = 1) of `focused_rect` among the given layout positions.
    ///
    /// Candidates must be strictly in the requested direction and share
    /// vertical space with the focused window. Among valid candidates the
    /// one with the most vertical overlap wins (ties broken by distance).
    fn find_spatial_neighbor(
        &self,
        positions: &[(usize, Rect)],
        focused: &Rect,
        direction: i32,
    ) -> Option<usize> {
        positions
            .iter()
            .filter(|(_, r)| {
                if direction > 0 {
                    r.center_x() > focused.center_x()
                } else {
                    r.center_x() < focused.center_x()
                }
            })
            .filter(|(_, r)| focused.vertical_overlap(r) > 0)
            .max_by_key(|(_, r)| {
                let overlap = focused.vertical_overlap(r);
                let dist = (r.center_x() - focused.center_x()).abs();
                // Prefer more overlap, then closer distance.
                (overlap, -dist)
            })
            .map(|(h, _)| *h)
    }

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
        self.update_border();
    }

    fn toggle_monocle(&mut self) {
        if self.monitors.is_empty() {
            return;
        }
        let idx = self.focused_monitor;
        self.monitors[idx].monocle = !self.monitors[idx].monocle;
        self.apply_layout_on(idx);
        self.update_border();
    }

    fn retile_all(&mut self) {
        for i in 0..self.monitors.len() {
            self.apply_layout_on(i);
        }
    }

    fn apply_layout_on(&mut self, monitor_idx: usize) {
        self.prune_stale_handles(monitor_idx);
        self.applying_layout = true;

        let Some(state) = self.monitors.get(monitor_idx) else {
            self.applying_layout = false;
            return;
        };
        if state.monocle {
            // In monocle mode, the focused window fills the work area.
            if let Some(hwnd) = self.focused_window
                && state.workspace.contains(hwnd)
            {
                let gap = self.layout.gap;
                let area = Rect::new(
                    state.work_area.x + gap,
                    state.work_area.y + gap,
                    (state.work_area.width - gap * 2).max(1),
                    (state.work_area.height - gap * 2).max(1),
                );
                let window = Window::from_raw(hwnd);
                if let Err(e) = window.set_rect(&area) {
                    eprintln!("Failed to position window 0x{hwnd:X}: {e}");
                }
                window.invalidate();
                self.applying_layout = false;
                return;
            }
        }
        let positions = state
            .workspace
            .compute_layout(&self.layout, &state.work_area);
        for (hwnd, rect) in &positions {
            let window = Window::from_raw(*hwnd);
            if let Err(e) = window.set_rect(rect) {
                eprintln!("Failed to position window 0x{hwnd:X}: {e}");
            }
            window.invalidate();
        }
        self.applying_layout = false;
    }

    /// Removes handles from the workspace that are no longer valid windows.
    ///
    /// Some windows (especially UWP and WinUI apps) may close without
    /// firing a reliable `EVENT_OBJECT_DESTROY`. This ensures stale
    /// handles don't leave phantom slots in the layout.
    fn prune_stale_handles(&mut self, monitor_idx: usize) {
        let Some(state) = self.monitors.get_mut(monitor_idx) else {
            return;
        };
        let stale: Vec<usize> = state
            .workspace
            .handles()
            .iter()
            .copied()
            .filter(|&hwnd| !Window::from_raw(hwnd).is_visible())
            .collect();
        for hwnd in stale {
            state.workspace.remove(hwnd);
        }
    }

    fn update_border(&self) {
        let Some(border) = &self.border else {
            return;
        };
        let Some(hwnd) = self.focused_window else {
            border.hide();
            return;
        };
        let window = Window::from_raw(hwnd);
        let Ok(rect) = window.rect() else {
            return;
        };
        let Some(mon) = self.monitors.get(self.focused_monitor) else {
            return;
        };
        let is_monocle = mon.monocle;
        let hex = if is_monocle {
            &self.border_config.monocle
        } else {
            &self.border_config.focused
        };
        let color = Color::from_hex(hex).unwrap_or(Color {
            r: 0,
            g: 0xB4,
            b: 0xD8,
        });
        border.show(&rect, color, self.border_config.width);
    }

    fn hide_border(&self) {
        if let Some(border) = &self.border {
            border.hide();
        }
    }

    /// Re-assigns a window to the correct monitor after it was moved.
    ///
    /// If the window moved to a different monitor, it is removed from
    /// the old workspace and added to the new one, then both monitors
    /// are re-tiled.
    fn reassign_monitor(&mut self, hwnd: usize) {
        let old = self.owning_monitor(hwnd);
        let new = self.monitor_index_for(hwnd);

        match (old, new) {
            (Some(from), Some(to)) if from != to => {
                self.monitors[from].workspace.remove(hwnd);
                self.monitors[to].workspace.add(hwnd);
                self.apply_layout_on(from);
                self.apply_layout_on(to);
            }
            (Some(idx), Some(_)) => {
                // Same monitor — just re-tile to snap it back.
                self.apply_layout_on(idx);
            }
            (None, Some(to)) if self.is_tileable(hwnd) => {
                // Window wasn't tracked but appeared on a monitor.
                self.monitors[to].workspace.add(hwnd);
                self.apply_layout_on(to);
            }
            _ => {}
        }
    }

    fn monitor_index_for(&self, hwnd: usize) -> Option<usize> {
        let mid = monitor::monitor_id_for_window(hwnd);
        self.monitors.iter().position(|m| m.id == mid)
    }

    fn owning_monitor(&self, hwnd: usize) -> Option<usize> {
        self.monitors
            .iter()
            .position(|m| m.workspace.contains(hwnd))
    }
}

/// Appends a diagnostic line to a temp log file.
///
/// Used to trace window add/remove operations while the daemon runs
/// detached (no stderr). Remove once the bug is diagnosed.
fn log(args: std::fmt::Arguments<'_>) {
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(r"C:\Users\jmelo\mosaico_debug.log")
    {
        let _ = writeln!(f, "{args}");
    }
}
