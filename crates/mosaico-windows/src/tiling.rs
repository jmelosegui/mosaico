use mosaico_core::action::Direction;
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
                    let w = Window::from_raw(*hwnd);
                    let title = w.title().unwrap_or_default();
                    let class = w.class().unwrap_or_default();
                    mosaico_core::log_info!(
                        "+add 0x{:X} [{}] \"{}\" to mon {} (now {})",
                        hwnd,
                        class,
                        title,
                        idx,
                        self.monitors[idx].workspace.len()
                    );
                    self.apply_layout_on(idx);
                }
            }
            WindowEvent::Destroyed { hwnd } | WindowEvent::Minimized { hwnd } => {
                if let Some(idx) = self.owning_monitor(*hwnd)
                    && self.monitors[idx].workspace.remove(*hwnd)
                {
                    mosaico_core::log_info!(
                        "-del 0x{:X} from mon {} (now {})",
                        hwnd,
                        idx,
                        self.monitors[idx].workspace.len()
                    );
                    self.apply_layout_on(idx);
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
            Action::Focus(dir) => self.focus_direction(*dir),
            Action::Move(dir) => self.move_direction(*dir),
            Action::Retile => self.retile_all(),
            Action::ToggleMonocle => self.toggle_monocle(),
            Action::CloseFocused => self.close_focused(),
        }
    }

    /// Returns the total number of managed windows.
    pub fn window_count(&self) -> usize {
        self.monitors.iter().map(|m| m.workspace.len()).sum()
    }

    /// Applies a new layout and border config, then retiles all windows.
    pub fn reload_config(&mut self, config: &mosaico_core::config::Config) {
        self.layout = BspLayout {
            gap: config.layout.gap,
            ratio: config.layout.ratio,
        };
        self.border_config = config.borders.clone();
        self.retile_all();
        self.update_border();
    }

    /// Replaces the window rules used for managing new windows.
    pub fn reload_rules(&mut self, rules: Vec<WindowRule>) {
        self.rules = rules;
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

    /// Focuses a window in the given spatial direction.
    ///
    /// Left/Right: horizontal neighbor, overflows to adjacent monitor.
    /// Up/Down: vertical neighbor, stops at boundary.
    fn focus_direction(&mut self, dir: Direction) {
        if self.monitors.is_empty() {
            return;
        }
        match dir {
            Direction::Left | Direction::Right => match self.resolve_horizontal_target(dir) {
                Some(SpatialTarget::Neighbor(hwnd)) => {
                    self.focused_window = Some(hwnd);
                    Window::from_raw(hwnd).set_foreground();
                    self.update_border();
                }
                Some(SpatialTarget::AdjacentMonitor(idx)) => {
                    self.focused_monitor = idx;
                    if let Some(hwnd) = self.find_entry_window(idx, dir) {
                        self.focused_window = Some(hwnd);
                        Window::from_raw(hwnd).set_foreground();
                        self.update_border();
                    }
                }
                None => {}
            },
            Direction::Up | Direction::Down => {
                if let Some(neighbor) = self.find_same_monitor_neighbor(dir) {
                    self.focused_window = Some(neighbor);
                    Window::from_raw(neighbor).set_foreground();
                    self.update_border();
                }
            }
        }
    }

    /// Moves (swaps) the focused window in the given spatial direction.
    ///
    /// Left/Right: horizontal swap, overflows to adjacent monitor.
    /// Up/Down: vertical swap, stops at boundary.
    fn move_direction(&mut self, dir: Direction) {
        if self.monitors.is_empty() {
            return;
        }
        let Some(hwnd) = self.focused_window else {
            return;
        };
        match dir {
            Direction::Left | Direction::Right => {
                match self.resolve_horizontal_target(dir) {
                    Some(SpatialTarget::Neighbor(neighbor)) => {
                        let ws = &self.monitors[self.focused_monitor].workspace;
                        let Some(a) = ws.index_of(hwnd) else { return };
                        let Some(b) = ws.index_of(neighbor) else {
                            return;
                        };
                        self.swap_and_retile(a, b);
                    }
                    Some(SpatialTarget::AdjacentMonitor(target)) => {
                        // Insert at position 0 when entering from the left
                        // so the window takes the leftmost BSP slot; append
                        // when entering from the right.
                        let source = self.focused_monitor;
                        self.monitors[source].workspace.remove(hwnd);
                        if dir == Direction::Right {
                            self.monitors[target].workspace.insert(0, hwnd);
                        } else {
                            self.monitors[target].workspace.add(hwnd);
                        }
                        self.apply_layout_on(source);
                        self.apply_layout_on(target);
                        self.focused_monitor = target;
                        self.update_border();
                    }
                    None => {}
                }
            }
            Direction::Up | Direction::Down => {
                let ws = &self.monitors[self.focused_monitor].workspace;
                let Some(idx) = ws.index_of(hwnd) else {
                    return;
                };
                if let Some(neighbor) = self.find_same_monitor_neighbor(dir)
                    && let Some(other) = self.monitors[self.focused_monitor]
                        .workspace
                        .index_of(neighbor)
                {
                    self.swap_and_retile(idx, other);
                }
            }
        }
    }

    /// Swaps two windows by workspace index, re-tiles, and updates the border.
    fn swap_and_retile(&mut self, a: usize, b: usize) {
        self.monitors[self.focused_monitor].workspace.swap(a, b);
        self.apply_layout_on(self.focused_monitor);
        self.update_border();
    }

    /// Finds the spatial neighbor for the focused window on the current
    /// monitor. Returns `None` at boundaries.
    fn find_same_monitor_neighbor(&self, dir: Direction) -> Option<usize> {
        let focused_hwnd = self.focused_window?;
        let state = &self.monitors[self.focused_monitor];
        let positions = state
            .workspace
            .compute_layout(&self.layout, &state.work_area);
        let focused_rect = positions
            .iter()
            .find(|(h, _)| *h == focused_hwnd)
            .map(|(_, r)| *r)?;
        mosaico_core::spatial::find_neighbor(&positions, &focused_rect, dir)
    }

    /// Resolves the spatial target for a left/right action.
    ///
    /// Looks for a neighbor on the same monitor first. If none exists,
    /// looks for a monitor physically in the requested direction (no
    /// wrapping).
    fn resolve_horizontal_target(&self, dir: Direction) -> Option<SpatialTarget> {
        let focused_hwnd = self.focused_window?;
        let positive = matches!(dir, Direction::Right | Direction::Down);

        let state = &self.monitors[self.focused_monitor];
        let positions = state
            .workspace
            .compute_layout(&self.layout, &state.work_area);

        mosaico_core::log_debug!(
            "resolve_horizontal dir={} mon={} focused=0x{:X} windows={}",
            dir,
            self.focused_monitor,
            focused_hwnd,
            positions.len()
        );
        for (h, r) in &positions {
            mosaico_core::log_debug!(
                "  pos 0x{:X}: ({},{} {}x{}) center_x={}",
                h,
                r.x,
                r.y,
                r.width,
                r.height,
                r.center_x()
            );
        }

        let focused_rect = positions
            .iter()
            .find(|(h, _)| *h == focused_hwnd)
            .map(|(_, r)| *r)?;

        mosaico_core::log_debug!(
            "  focused_rect: ({},{} {}x{}) center_x={}",
            focused_rect.x,
            focused_rect.y,
            focused_rect.width,
            focused_rect.height,
            focused_rect.center_x()
        );

        if let Some(neighbor) = mosaico_core::spatial::find_neighbor(&positions, &focused_rect, dir)
        {
            mosaico_core::log_debug!("  -> Neighbor 0x{:X}", neighbor);
            return Some(SpatialTarget::Neighbor(neighbor));
        }

        // No horizontal neighbor — check if a monitor exists in the
        // requested direction (no wrapping).
        let current_cx = state.work_area.center_x();
        let adjacent = self
            .monitors
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != self.focused_monitor)
            .filter(|(_, m)| {
                if positive {
                    m.work_area.center_x() > current_cx
                } else {
                    m.work_area.center_x() < current_cx
                }
            })
            .min_by_key(|(_, m)| (m.work_area.center_x() - current_cx).abs())
            .map(|(i, _)| i);

        match adjacent {
            Some(idx) => {
                mosaico_core::log_debug!("  -> AdjacentMonitor {}", idx);
                Some(SpatialTarget::AdjacentMonitor(idx))
            }
            None => {
                mosaico_core::log_debug!("  -> None (no monitor in dir {})", dir);
                None
            }
        }
    }

    fn find_entry_window(&self, monitor_idx: usize, dir: Direction) -> Option<usize> {
        let state = &self.monitors[monitor_idx];
        let positions = state
            .workspace
            .compute_layout(&self.layout, &state.work_area);
        mosaico_core::spatial::find_entry(&positions, dir)
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
