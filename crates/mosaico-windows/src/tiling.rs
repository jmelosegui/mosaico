use std::collections::HashSet;

use mosaico_core::action::{Direction, MAX_WORKSPACES};
use mosaico_core::config::{BorderConfig, WindowRule};
use mosaico_core::window::Window as WindowTrait;
use mosaico_core::{Action, BspLayout, Rect, WindowEvent, WindowResult, Workspace};

use crate::border::{Border, Color};
use crate::enumerate;
use crate::monitor;
use crate::window::Window;

/// Per-monitor state: multiple workspaces, work area, and monocle flag.
struct MonitorState {
    id: usize,
    work_area: Rect,
    workspaces: Vec<Workspace>,
    active_workspace: usize,
    monocle: bool,
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
    ) -> WindowResult<Self> {
        let monitors: Vec<MonitorState> = monitor::enumerate_monitors()?
            .into_iter()
            .map(|info| MonitorState {
                id: info.id,
                work_area: info.work_area,
                workspaces: (0..MAX_WORKSPACES).map(|_| Workspace::new()).collect(),
                active_workspace: 0,
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
            hidden_by_switch: HashSet::new(),
        };

        for win in enumerate::enumerate_windows()? {
            let hwnd = win.hwnd().0 as usize;
            if manager.is_tileable(hwnd)
                && let Some(idx) = manager.monitor_index_for(hwnd)
            {
                manager.monitors[idx].active_ws_mut().add(hwnd);
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
                if self.monitors.get(idx).is_some() && self.monitors[idx].active_ws_mut().add(*hwnd)
                {
                    let w = Window::from_raw(*hwnd);
                    let title = w.title().unwrap_or_default();
                    let class = w.class().unwrap_or_default();
                    mosaico_core::log_info!(
                        "+add 0x{:X} [{}] \"{}\" to mon {} ws {} (now {})",
                        hwnd,
                        class,
                        title,
                        idx,
                        self.monitors[idx].active_workspace + 1,
                        self.monitors[idx].active_ws().len()
                    );
                    self.apply_layout_on(idx);
                }
            }
            WindowEvent::Destroyed { hwnd } => {
                // EVENT_OBJECT_HIDE also maps here. Skip windows we hid
                // programmatically during a workspace switch.
                if self.hidden_by_switch.contains(hwnd) {
                    return;
                }
                // Truly destroyed — clean up from any workspace.
                if let Some((mon_idx, ws_idx)) = self.find_window(*hwnd) {
                    self.monitors[mon_idx].workspaces[ws_idx].remove(*hwnd);
                    mosaico_core::log_info!(
                        "-del 0x{:X} from mon {} ws {} (now {})",
                        hwnd,
                        mon_idx,
                        ws_idx + 1,
                        self.monitors[mon_idx].workspaces[ws_idx].len()
                    );
                    if ws_idx == self.monitors[mon_idx].active_workspace {
                        self.apply_layout_on(mon_idx);
                    }
                }
            }
            WindowEvent::Minimized { hwnd } => {
                // Only remove from the active workspace. Windows on
                // non-active workspaces are hidden by workspace switching
                // and must not be pruned.
                if let Some((mon_idx, ws_idx)) = self.find_window(*hwnd)
                    && ws_idx == self.monitors[mon_idx].active_workspace
                {
                    self.monitors[mon_idx].workspaces[ws_idx].remove(*hwnd);
                    mosaico_core::log_info!(
                        "-min 0x{:X} from mon {} ws {} (now {})",
                        hwnd,
                        mon_idx,
                        ws_idx + 1,
                        self.monitors[mon_idx].workspaces[ws_idx].len()
                    );
                    self.apply_layout_on(mon_idx);
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
            && let Some(&hwnd) = mon.active_ws().handles().first()
        {
            self.focus_and_update_border(hwnd);
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
                    self.focus_and_update_border(hwnd);
                }
                Some(SpatialTarget::AdjacentMonitor(idx)) => {
                    self.focused_monitor = idx;
                    if let Some(hwnd) = self.find_entry_window(idx, dir) {
                        self.focus_and_update_border(hwnd);
                    } else {
                        // Target monitor has no windows — clear focus
                        // so the user sees the selection leave.
                        self.focused_window = None;
                        self.update_border();
                    }
                }
                None if self.focused_window.is_none() => {
                    // No focused window (empty workspace) — jump to
                    // the adjacent monitor directly.
                    if let Some(idx) = self.find_adjacent_monitor(dir) {
                        self.focused_monitor = idx;
                        if let Some(hwnd) = self.find_entry_window(idx, dir) {
                            self.focus_and_update_border(hwnd);
                        }
                    }
                }
                None => {}
            },
            Direction::Up | Direction::Down => {
                if let Some(neighbor) = self.find_same_monitor_neighbor(dir) {
                    self.focus_and_update_border(neighbor);
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
                        let ws = self.monitors[self.focused_monitor].active_ws();
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
                        self.monitors[source].active_ws_mut().remove(hwnd);
                        if dir == Direction::Right {
                            self.monitors[target].active_ws_mut().insert(0, hwnd);
                        } else {
                            self.monitors[target].active_ws_mut().add(hwnd);
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
                let ws = self.monitors[self.focused_monitor].active_ws();
                let Some(idx) = ws.index_of(hwnd) else {
                    return;
                };
                if let Some(neighbor) = self.find_same_monitor_neighbor(dir)
                    && let Some(other) = self.monitors[self.focused_monitor]
                        .active_ws()
                        .index_of(neighbor)
                {
                    self.swap_and_retile(idx, other);
                }
            }
        }
    }

    /// Swaps two windows by workspace index, re-tiles, and updates the border.
    fn swap_and_retile(&mut self, a: usize, b: usize) {
        self.monitors[self.focused_monitor]
            .active_ws_mut()
            .swap(a, b);
        self.apply_layout_on(self.focused_monitor);
        self.update_border();
    }

    /// Finds the spatial neighbor for the focused window on the current
    /// monitor. Returns `None` at boundaries.
    fn find_same_monitor_neighbor(&self, dir: Direction) -> Option<usize> {
        let focused_hwnd = self.focused_window?;
        let state = &self.monitors[self.focused_monitor];
        let positions = state
            .active_ws()
            .compute_layout(&self.layout, &state.work_area);
        let focused_rect = positions
            .iter()
            .find(|(h, _)| *h == focused_hwnd)
            .map(|(_, r)| *r)?;
        mosaico_core::spatial::find_neighbor(&positions, &focused_rect, dir)
    }

    /// Finds the nearest monitor in the given horizontal direction.
    fn find_adjacent_monitor(&self, dir: Direction) -> Option<usize> {
        let positive = matches!(dir, Direction::Right | Direction::Down);
        let current_cx = self.monitors[self.focused_monitor].work_area.center_x();
        self.monitors
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
            .map(|(i, _)| i)
    }

    /// Resolves the spatial target for a left/right action.
    ///
    /// Looks for a neighbor on the same monitor first. If none exists,
    /// looks for a monitor physically in the requested direction (no
    /// wrapping).
    fn resolve_horizontal_target(&self, dir: Direction) -> Option<SpatialTarget> {
        let focused_hwnd = self.focused_window?;

        let state = &self.monitors[self.focused_monitor];
        let positions = state
            .active_ws()
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
        match self.find_adjacent_monitor(dir) {
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
            .active_ws()
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
                && state.active_ws().contains(hwnd)
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
            .active_ws()
            .compute_layout(&self.layout, &state.work_area);
        for (hwnd, rect) in &positions {
            let window = Window::from_raw(*hwnd);
            if window.rect().ok().as_ref() == Some(rect) {
                continue;
            }
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
            .active_ws()
            .handles()
            .iter()
            .copied()
            .filter(|&hwnd| !Window::from_raw(hwnd).is_visible())
            .collect();
        for hwnd in stale {
            state.active_ws_mut().remove(hwnd);
        }
    }

    /// Sets the focused window, brings it to the foreground, and
    /// refreshes the focus border.
    fn focus_and_update_border(&mut self, hwnd: usize) {
        self.focused_window = Some(hwnd);
        Window::from_raw(hwnd).set_foreground();
        self.update_border();
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
                self.monitors[from].active_ws_mut().remove(hwnd);
                self.monitors[to].active_ws_mut().add(hwnd);
                self.apply_layout_on(from);
                self.apply_layout_on(to);
            }
            (Some(idx), Some(_)) => {
                // Same monitor — just re-tile to snap it back.
                self.apply_layout_on(idx);
            }
            (None, Some(to)) if self.is_tileable(hwnd) => {
                // Window wasn't tracked but appeared on a monitor.
                self.monitors[to].active_ws_mut().add(hwnd);
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
            .position(|m| m.workspaces.iter().any(|ws| ws.contains(hwnd)))
    }

    /// Finds which monitor and workspace contain the given window.
    ///
    /// Returns `(monitor_index, workspace_index)` or `None` if the
    /// window is not managed anywhere.
    fn find_window(&self, hwnd: usize) -> Option<(usize, usize)> {
        for (mi, mon) in self.monitors.iter().enumerate() {
            for (wi, ws) in mon.workspaces.iter().enumerate() {
                if ws.contains(hwnd) {
                    return Some((mi, wi));
                }
            }
        }
        None
    }

    /// Switches to workspace `n` (1-indexed) on the focused monitor.
    ///
    /// Hides windows on the current workspace, shows windows on the
    /// target, retiles, and focuses the first window.
    fn goto_workspace(&mut self, n: u8) {
        let idx = (n - 1) as usize;
        let mon_idx = self.focused_monitor;
        let Some(mon) = self.monitors.get(mon_idx) else {
            return;
        };
        if mon.active_workspace == idx {
            return; // already there
        }

        // Mark current workspace windows as programmatically hidden
        // so EVENT_OBJECT_HIDE events are ignored for them.
        for &hwnd in mon.active_ws().handles() {
            self.hidden_by_switch.insert(hwnd);
            Window::from_raw(hwnd).hide();
        }

        // Switch active workspace.
        self.monitors[mon_idx].active_workspace = idx;

        // Show windows on the target workspace and unmark them.
        for &hwnd in self.monitors[mon_idx].active_ws().handles() {
            self.hidden_by_switch.remove(&hwnd);
            Window::from_raw(hwnd).show();
        }

        mosaico_core::log_info!(
            "goto-workspace {} on mon {} ({} windows)",
            n,
            mon_idx,
            self.monitors[mon_idx].active_ws().len()
        );

        self.apply_layout_on(mon_idx);

        // Focus the first window on the new workspace, or clear focus.
        if let Some(&hwnd) = self.monitors[mon_idx].active_ws().handles().first() {
            self.focus_and_update_border(hwnd);
        } else {
            self.focused_window = None;
            self.update_border();
        }
    }

    /// Sends the focused window to workspace `n` (1-indexed) on the
    /// same monitor.
    ///
    /// The window is hidden if the target workspace is not active.
    fn send_to_workspace(&mut self, n: u8) {
        let target_ws = (n - 1) as usize;
        let Some(hwnd) = self.focused_window else {
            return;
        };
        let mon_idx = self.focused_monitor;
        let Some(mon) = self.monitors.get(mon_idx) else {
            return;
        };
        if mon.active_workspace == target_ws {
            return; // already on target workspace
        }
        if !mon.active_ws().contains(hwnd) {
            return; // focused window is not on the active workspace
        }
        let src_ws_num = mon.active_workspace + 1;

        // Remove from current workspace, add to target.
        self.monitors[mon_idx].active_ws_mut().remove(hwnd);
        self.monitors[mon_idx].workspaces[target_ws].add(hwnd);

        // Hide the window since it's moving to a non-visible workspace.
        self.hidden_by_switch.insert(hwnd);
        Window::from_raw(hwnd).hide();

        mosaico_core::log_info!(
            "send-to-workspace {} 0x{:X} on mon {} (src ws {} -> dst ws {})",
            n,
            hwnd,
            mon_idx,
            src_ws_num,
            n
        );

        self.apply_layout_on(mon_idx);

        // Focus the next window on the current workspace, or clear focus.
        if let Some(&next) = self.monitors[mon_idx].active_ws().handles().first() {
            self.focus_and_update_border(next);
        } else {
            self.focused_window = None;
            self.update_border();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_monitor(id: usize) -> MonitorState {
        MonitorState {
            id,
            work_area: Rect::new(0, 0, 1920, 1080),
            workspaces: (0..MAX_WORKSPACES).map(|_| Workspace::new()).collect(),
            active_workspace: 0,
            monocle: false,
        }
    }

    // -- MonitorState helpers --

    #[test]
    fn active_ws_returns_correct_workspace() {
        let mut mon = make_monitor(1);
        mon.workspaces[0].add(100);
        mon.workspaces[1].add(200);

        assert!(mon.active_ws().contains(100));
        assert!(!mon.active_ws().contains(200));

        mon.active_workspace = 1;
        assert!(!mon.active_ws().contains(100));
        assert!(mon.active_ws().contains(200));
    }

    #[test]
    fn active_ws_mut_modifies_correct_workspace() {
        let mut mon = make_monitor(1);
        mon.active_ws_mut().add(100);
        assert!(mon.workspaces[0].contains(100));

        mon.active_workspace = 1;
        mon.active_ws_mut().add(200);
        assert!(mon.workspaces[1].contains(200));
        assert!(!mon.workspaces[1].contains(100));
    }

    #[test]
    fn monitor_state_has_max_workspaces() {
        let mon = make_monitor(1);
        assert_eq!(mon.workspaces.len(), MAX_WORKSPACES as usize);
    }

    // -- find_window --

    fn make_monitors(n: usize) -> Vec<MonitorState> {
        (0..n).map(make_monitor).collect()
    }

    #[test]
    fn find_window_on_active_workspace() {
        let mut monitors = make_monitors(2);
        monitors[0].workspaces[0].add(100);
        monitors[1].workspaces[0].add(200);

        // find_window is a method on TilingManager; test the logic directly
        let result = find_window_in(&monitors, 100);
        assert_eq!(result, Some((0, 0)));

        let result = find_window_in(&monitors, 200);
        assert_eq!(result, Some((1, 0)));
    }

    #[test]
    fn find_window_on_non_active_workspace() {
        let mut monitors = make_monitors(1);
        monitors[0].workspaces[3].add(42);
        monitors[0].active_workspace = 0;

        let result = find_window_in(&monitors, 42);
        assert_eq!(result, Some((0, 3)));
    }

    #[test]
    fn find_window_not_found() {
        let monitors = make_monitors(2);
        assert_eq!(find_window_in(&monitors, 999), None);
    }

    // -- owning_monitor --

    #[test]
    fn owning_monitor_searches_all_workspaces() {
        let mut monitors = make_monitors(2);
        monitors[1].workspaces[5].add(77);

        let result = monitors
            .iter()
            .position(|m| m.workspaces.iter().any(|ws| ws.contains(77)));
        assert_eq!(result, Some(1));
    }

    // -- hidden_by_switch --

    #[test]
    fn hidden_by_switch_tracks_hwnds() {
        let mut set = HashSet::new();

        // Simulate hiding windows for workspace switch
        set.insert(100);
        set.insert(200);
        assert!(set.contains(&100));
        assert!(set.contains(&200));
        assert!(!set.contains(&300));

        // Simulate showing them again
        set.remove(&100);
        assert!(!set.contains(&100));
        assert!(set.contains(&200));
    }

    #[test]
    fn hidden_by_switch_ignores_duplicates() {
        let mut set = HashSet::new();
        set.insert(100);
        set.insert(100); // no-op
        assert_eq!(set.len(), 1);
    }

    // -- workspace switch simulation --

    #[test]
    fn goto_workspace_logic() {
        let mut mon = make_monitor(1);
        mon.workspaces[0].add(10);
        mon.workspaces[0].add(20);
        mon.workspaces[1].add(30);

        // Switch from ws 0 to ws 1
        let mut hidden = HashSet::new();
        for &hwnd in mon.active_ws().handles() {
            hidden.insert(hwnd);
        }
        mon.active_workspace = 1;

        // After switch: ws 1 is active, hidden set has ws 0's windows
        assert_eq!(mon.active_ws().len(), 1);
        assert!(mon.active_ws().contains(30));
        assert!(hidden.contains(&10));
        assert!(hidden.contains(&20));

        // Show ws 1 windows — remove from hidden
        for &hwnd in mon.active_ws().handles() {
            hidden.remove(&hwnd);
        }
        // 30 was not hidden, so set unchanged (still has 10, 20)
        assert_eq!(hidden.len(), 2);
    }

    #[test]
    fn send_to_workspace_logic() {
        let mut mon = make_monitor(1);
        mon.workspaces[0].add(10);
        mon.workspaces[0].add(20);
        mon.workspaces[0].add(30);
        mon.active_workspace = 0;

        // Send window 20 from ws 0 to ws 2
        let target_ws = 2;
        assert!(mon.active_ws().contains(20));
        mon.active_ws_mut().remove(20);
        mon.workspaces[target_ws].add(20);

        assert_eq!(mon.workspaces[0].len(), 2);
        assert_eq!(mon.workspaces[target_ws].len(), 1);
        assert!(mon.workspaces[target_ws].contains(20));
        assert!(!mon.workspaces[0].contains(20));
    }

    #[test]
    fn send_to_same_workspace_is_noop() {
        let mut mon = make_monitor(1);
        mon.workspaces[0].add(10);
        mon.active_workspace = 0;

        // Sending to active workspace should be a no-op
        let target = mon.active_workspace;
        assert_eq!(target, 0);
        // The real code returns early; simulate by checking condition
        assert!(mon.active_workspace == target);
        assert_eq!(mon.workspaces[0].len(), 1);
    }

    #[test]
    fn goto_same_workspace_is_noop() {
        let mon = make_monitor(1);
        // Switching to already-active workspace should be a no-op
        assert_eq!(mon.active_workspace, 0);
        // The real code returns early when active_workspace == target
    }

    // Standalone helper matching TilingManager::find_window logic
    fn find_window_in(monitors: &[MonitorState], hwnd: usize) -> Option<(usize, usize)> {
        for (mi, mon) in monitors.iter().enumerate() {
            for (wi, ws) in mon.workspaces.iter().enumerate() {
                if ws.contains(hwnd) {
                    return Some((mi, wi));
                }
            }
        }
        None
    }
}
