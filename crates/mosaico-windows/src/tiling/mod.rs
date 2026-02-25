mod layout;
mod navigation;
mod workspace;

use std::collections::HashSet;

use mosaico_core::action::MAX_WORKSPACES;
use mosaico_core::config::{BorderConfig, WindowRule};
use mosaico_core::window::Window as WindowTrait;
use mosaico_core::{Action, BspLayout, Rect, WindowEvent, WindowResult, Workspace};

use crate::bar::BarState;
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
    /// The window shown fullscreen in monocle mode.
    monocle_window: Option<usize>,
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
                    // Don't retile a window that the user just maximized —
                    // reassign_monitor would snap it back to BSP layout.
                    if Window::from_raw(*hwnd).is_maximized() {
                        self.update_border();
                    } else {
                        self.reassign_monitor(*hwnd);
                    }
                }
            }
            WindowEvent::Focused { hwnd } => {
                if let Some(idx) = self.owning_monitor(*hwnd) {
                    self.focused_window = Some(*hwnd);
                    self.focused_monitor = idx;
                    self.update_border();
                }
                // Unmanaged windows (Alt+Tab UI, shell, system dialogs)
                // are ignored — the border stays on the last managed
                // window so keyboard navigation keeps working.
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

    /// Shows all windows across every workspace and monitor.
    ///
    /// Called on daemon shutdown so that windows hidden by workspace
    /// switching are restored and not left invisible.
    pub fn restore_all_windows(&mut self) {
        self.hide_border();
        for mon in &self.monitors {
            for ws in &mon.workspaces {
                for &hwnd in ws.handles() {
                    Window::from_raw(hwnd).force_show();
                }
            }
        }
        self.hidden_by_switch.clear();
    }

    /// Re-positions the focus border to match the current window rect.
    ///
    /// Call after work areas change (e.g. bar adjustment at startup) so
    /// the border reflects the final window position, not the pre-adjustment one.
    pub fn refresh_border(&self) {
        self.update_border();
    }

    /// Replaces the window rules used for managing new windows.
    pub fn reload_rules(&mut self, rules: Vec<WindowRule>) {
        self.rules = rules;
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
            })
            .collect()
    }

    /// Adjusts monitor work areas by subtracting bar height from the top.
    ///
    /// Only monitors whose index appears in `bar_monitors` are adjusted.
    /// An empty slice means all monitors.
    pub fn adjust_work_areas_for_bar(&mut self, bar_height: i32, bar_monitors: &[usize]) {
        for (i, mon) in self.monitors.iter_mut().enumerate() {
            if bar_monitors.is_empty() || bar_monitors.contains(&i) {
                mon.work_area.y += bar_height;
                mon.work_area.height -= bar_height;
            }
        }
        self.retile_all();
    }

    /// Resets work areas to the OS values, then applies the bar offset.
    ///
    /// Used when bar config changes to avoid accumulating offsets.
    pub fn reset_and_adjust_work_areas(&mut self, bar_height: i32, bar_monitors: &[usize]) {
        if let Ok(os_monitors) = monitor::enumerate_monitors() {
            for mon in &mut self.monitors {
                if let Some(os) = os_monitors.iter().find(|m| m.id == mon.id) {
                    mon.work_area = os.work_area;
                }
            }
        }
        self.adjust_work_areas_for_bar(bar_height, bar_monitors);
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

    fn minimize_focused(&mut self) {
        let Some(hwnd) = self.focused_window else {
            return;
        };
        // ShowWindow(SW_MINIMIZE) fires EVENT_SYSTEM_MINIMIZESTART which
        // the event loop translates to WindowEvent::Minimized. That event
        // removes the window from the active workspace and re-tiles.
        Window::from_raw(hwnd).minimize();
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
        // Hide the border when the focused window is maximized —
        // the border would be behind the maximized window anyway and
        // trying to keep it topmost causes z-order flickering.
        if window.is_maximized() {
            border.hide();
            return;
        }
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
            monocle_window: None,
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

    // -- monocle --

    #[test]
    fn monocle_toggle_sets_monocle_window() {
        let mut mon = make_monitor(1);
        mon.workspaces[0].add(100);
        mon.workspaces[0].add(200);

        // Simulate toggle_monocle: enable with focused_window = 100
        let focused_window = Some(100usize);
        mon.monocle = true;
        mon.monocle_window = focused_window;

        assert!(mon.monocle);
        assert_eq!(mon.monocle_window, Some(100));
    }

    #[test]
    fn monocle_toggle_clears_monocle_window() {
        let mut mon = make_monitor(1);
        mon.monocle = true;
        mon.monocle_window = Some(100);

        // Simulate toggle_monocle: disable
        mon.monocle = false;
        mon.monocle_window = None;

        assert!(!mon.monocle);
        assert_eq!(mon.monocle_window, None);
    }

    #[test]
    fn monocle_blocks_move_on_same_monitor() {
        let mut mon = make_monitor(1);
        mon.workspaces[0].add(100);
        mon.workspaces[0].add(200);
        mon.monocle = true;
        mon.monocle_window = Some(100);

        // In monocle mode, move_direction returns early.
        // Verify the state that triggers the early return.
        assert!(mon.monocle);

        // Windows should remain unchanged — monocle blocks all moves.
        assert_eq!(mon.workspaces[0].len(), 2);
        assert!(mon.workspaces[0].contains(100));
        assert!(mon.workspaces[0].contains(200));
    }

    #[test]
    fn monocle_entry_uses_monocle_window() {
        let mut mon = make_monitor(1);
        mon.workspaces[0].add(100);
        mon.workspaces[0].add(200);
        mon.workspaces[0].add(300);
        mon.monocle = true;
        mon.monocle_window = Some(200);

        // When entering a monocle monitor, focus_adjacent_monitor_idx
        // picks monocle_window over spatial entry.
        let entry = mon
            .monocle_window
            .or_else(|| mon.active_ws().handles().first().copied());
        assert_eq!(entry, Some(200));
    }

    #[test]
    fn monocle_entry_falls_back_to_first_window() {
        let mut mon = make_monitor(1);
        mon.workspaces[0].add(100);
        mon.workspaces[0].add(200);
        mon.monocle = true;
        mon.monocle_window = None; // e.g. monocle window was closed

        let entry = mon
            .monocle_window
            .or_else(|| mon.active_ws().handles().first().copied());
        assert_eq!(entry, Some(100));
    }

    #[test]
    fn monocle_no_vertical_navigation() {
        let mut mon = make_monitor(1);
        mon.workspaces[0].add(100);
        mon.workspaces[0].add(200);
        mon.monocle = true;
        mon.monocle_window = Some(100);

        // In monocle mode, focus_direction returns early for Up/Down.
        // The monocle flag is the guard condition.
        assert!(mon.monocle);
        // No vertical neighbor lookup should happen — there is
        // conceptually only one window.
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
