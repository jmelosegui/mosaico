//! Event handling for the tiling manager.

use mosaico_core::WindowEvent;
use mosaico_core::window::Window as WindowTrait;

use crate::frame;
use crate::window::Window;

use super::TilingManager;

impl TilingManager {
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
                    frame::set_corner_preference(w.hwnd(), self.border_config.corner_style);
                    // Focus the new window before layout so monocle
                    // mode sizes the correct window.
                    self.focused_window = Some(*hwnd);
                    self.apply_layout_on(idx);
                    self.focus_and_update_border(*hwnd);
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
                    frame::reset_corner_preference(Window::from_raw(*hwnd).hwnd());
                    self.monitors[mon_idx].workspaces[ws_idx].remove(*hwnd);
                    mosaico_core::log_info!(
                        "-del 0x{:X} from mon {} ws {} (now {})",
                        hwnd,
                        mon_idx,
                        ws_idx + 1,
                        self.monitors[mon_idx].workspaces[ws_idx].len()
                    );
                    // Clear monocle if the monocle window was destroyed.
                    if self.monitors[mon_idx].monocle
                        && self.monitors[mon_idx].monocle_window == Some(*hwnd)
                    {
                        self.monitors[mon_idx].monocle = false;
                        self.monitors[mon_idx].monocle_window = None;
                    }
                    if ws_idx == self.monitors[mon_idx].active_workspace {
                        self.apply_layout_on(mon_idx);
                    }
                }
            }
            WindowEvent::Minimized { hwnd } => {
                // Programmatic minimize from workspace switch — ignore.
                if self.hidden_by_switch.contains(hwnd) {
                    return;
                }
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
                    let maximized = Window::from_raw(*hwnd).is_maximized();
                    if self.focused_window == Some(*hwnd) {
                        self.focused_maximized = maximized;
                    }
                    if maximized {
                        self.update_border();
                    } else {
                        self.reassign_monitor(*hwnd);
                    }
                }
            }
            WindowEvent::Focused { hwnd } => {
                if let Some(idx) = self.owning_monitor(*hwnd) {
                    // Check if the window is on a non-active workspace
                    // (e.g. user clicked a cloaked window's taskbar icon).
                    // Switch to that workspace so the window becomes visible.
                    if let Some((mon_idx, ws_idx)) = self.find_window(*hwnd)
                        && ws_idx != self.monitors[mon_idx].active_workspace
                    {
                        self.focused_monitor = mon_idx;
                        self.goto_workspace((ws_idx + 1) as u8);
                        // goto_workspace focuses the first window; refocus
                        // the one the user actually clicked.
                        self.focus_and_update_border(*hwnd);
                        return;
                    }
                    self.focused_window = Some(*hwnd);
                    self.focused_monitor = idx;
                    self.focused_maximized = Window::from_raw(*hwnd).is_maximized();
                    self.update_border();
                } else if let Some(owner) = Window::from_raw(*hwnd).owner()
                    && let Some(idx) = self.owning_monitor(owner)
                {
                    // An owned window (dialog, property sheet) got focus.
                    //
                    // Due to a Win32 race condition, dialogs created on
                    // a different thread may not have their owner set
                    // when EVENT_OBJECT_CREATE fires, causing them to
                    // pass is_app_window() and get tiled. Now that the
                    // owner is queryable, clean up if the dialog was
                    // incorrectly added.
                    if let Some((mon_idx, ws_idx)) = self.find_window(*hwnd) {
                        self.monitors[mon_idx].workspaces[ws_idx].remove(*hwnd);
                        mosaico_core::log_info!(
                            "-fix 0x{:X} (owned dialog removed from tiling)",
                            hwnd
                        );
                        self.apply_layout_on(mon_idx);
                    }
                    // Move the border to the owner so the user can see
                    // which application the dialog belongs to.
                    self.focused_window = Some(owner);
                    self.focused_monitor = idx;
                    self.focused_maximized = Window::from_raw(owner).is_maximized();
                    self.update_border();
                }
                // Unmanaged windows without a managed owner (Alt+Tab UI,
                // shell, system dialogs) are ignored — the border stays
                // on the last managed window so keyboard navigation
                // keeps working.
            }
            WindowEvent::LocationChanged { hwnd } => {
                // EVENT_OBJECT_LOCATIONCHANGE fires frequently (every
                // animation frame), so only react when the focused
                // window's maximized state changes. This catches
                // maximize/restore via the title-bar buttons which do
                // not fire EVENT_SYSTEM_MOVESIZEEND.
                if self.focused_window == Some(*hwnd) {
                    let maximized = Window::from_raw(*hwnd).is_maximized();
                    if maximized != self.focused_maximized {
                        self.focused_maximized = maximized;
                        if maximized {
                            // Entering maximize — hide border immediately.
                            self.update_border();
                        } else {
                            // Restoring from maximize — retile to snap
                            // the window back to its BSP position so the
                            // border doesn't flash before the animation
                            // finishes.
                            self.reassign_monitor(*hwnd);
                            self.update_border();
                        }
                    }
                }
            }
            WindowEvent::DisplayChanged | WindowEvent::WorkAreaChanged => {
                // Handled by the daemon loop, not here.
            }
            _ => {}
        }
    }
}
