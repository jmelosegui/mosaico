//! Window lookup and management helpers for the tiling manager.

use mosaico_core::window::Window as WindowTrait;

use crate::frame;
use crate::monitor;
use crate::window::Window;

use super::TilingManager;

/// Whether a window that Windows has created but not shown yet should
/// be moved into the slot it is going to occupy.
///
/// Monocle used to be excluded here, because `compute_positions` lays
/// out every window in the workspace while monocle only ever shows one,
/// so the rect it produced was not where the window would end up.
/// `pre_place` now asks `monocle_rect` instead on a monocle workspace,
/// so the destination is right either way and the caller no longer has
/// to know which mode it is in.
pub(super) fn should_pre_place(tracked: bool, passes_rules: bool) -> bool {
    !tracked && passes_rules
}

impl TilingManager {
    pub(super) fn is_tileable(&self, hwnd: usize) -> bool {
        Window::from_raw(hwnd).is_visible() && self.passes_tiling_rules(hwnd)
    }

    /// Everything `is_tileable` checks except visibility.
    ///
    /// Split out so the `Created` handler can evaluate a window that
    /// Windows has created but not shown yet. Such a window fails the
    /// visibility test by definition, yet it is exactly the window we
    /// want to position before its first frame reaches the screen.
    pub(super) fn passes_tiling_rules(&self, hwnd: usize) -> bool {
        let window = Window::from_raw(hwnd);
        if !window.is_app_window() || window.is_cloaked() {
            return false;
        }
        // When mosaico runs as a regular user, SetWindowPos silently fails
        // on windows owned by elevated processes (UIPI). Skip them.
        if !self.self_elevated && window.is_elevated() {
            let title = window.title().unwrap_or_default();
            let class = window.class().unwrap_or_default();
            mosaico_core::log_info!(
                "~skip 0x{:X} [{}] \"{}\" (elevated, mosaico is not)",
                hwnd,
                class,
                title,
            );
            return false;
        }
        let class = window.class().unwrap_or_default();
        let title = window.title().unwrap_or_default();
        mosaico_core::config::should_manage(&class, &title, &self.rules)
    }

    /// Re-assigns a window to the correct monitor after it was moved.
    ///
    /// If the window moved to a different monitor, it is removed from
    /// the old workspace and added to the new one, then both monitors
    /// are re-tiled.
    pub(super) fn reassign_monitor(&mut self, hwnd: usize) {
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

    pub(super) fn monitor_index_for(&self, hwnd: usize) -> Option<usize> {
        let mid = monitor::monitor_id_for_window(hwnd);
        self.monitors.iter().position(|m| m.id == mid)
    }

    pub(super) fn is_managed_on_active_workspace(&self, hwnd: usize) -> bool {
        self.monitors
            .get(self.focused_monitor)
            .is_some_and(|mon| mon.active_ws().contains(hwnd))
    }

    pub(super) fn owning_monitor(&self, hwnd: usize) -> Option<usize> {
        self.monitors
            .iter()
            .position(|m| m.workspaces.iter().any(|ws| ws.contains(hwnd)))
    }

    /// Finds which monitor and workspace contain the given window.
    ///
    /// Returns `(monitor_index, workspace_index)` or `None` if the
    /// window is not managed anywhere.
    pub(super) fn find_window(&self, hwnd: usize) -> Option<(usize, usize)> {
        for (mi, mon) in self.monitors.iter().enumerate() {
            for (wi, ws) in mon.workspaces.iter().enumerate() {
                if ws.contains(hwnd) {
                    return Some((mi, wi));
                }
            }
        }
        None
    }

    /// Adopts an untracked window into the tiling layout if it is tileable.
    ///
    /// Some applications (e.g. WPF-based apps like Visual Studio) never
    /// fire EVENT_OBJECT_CREATE for their main window. This method is
    /// called from fallback event handlers (TitleChanged, Focused) to
    /// pick up windows that the Created handler missed.
    pub(super) fn try_adopt(&mut self, hwnd: usize) {
        // Don't use adopt_rejected here — try_adopt is called from
        // TitleChanged (relaxed id_object filter) where child element
        // events share the parent hwnd and would poison the cache.
        if !self.is_tileable(hwnd) {
            return;
        }
        let Some(idx) = self.monitor_index_for(hwnd) else {
            return;
        };
        if !self.monitors[idx].active_ws_mut().add(hwnd) {
            return;
        }
        let w = Window::from_raw(hwnd);
        let title = w.title().unwrap_or_default();
        let class = w.class().unwrap_or_default();
        mosaico_core::log_info!(
            "+adopt 0x{:X} [{}] \"{}\" on mon {} ws {} (now {})",
            hwnd,
            class,
            title,
            idx,
            self.monitors[idx].active_workspace + 1,
            self.monitors[idx].active_ws().len()
        );
        frame::set_corner_preference(w.hwnd(), self.border_config.corner_style);
        self.apply_layout_on(idx);
    }

    /// Adds a window to the focused monitor's active workspace and focuses it.
    ///
    /// Shared by the `Created` and `Restored` event handlers.
    pub(super) fn add_and_focus(&mut self, hwnd: usize) {
        // Place new windows on the focused monitor so they appear
        // where the user is working, not wherever the OS spawns them.
        let idx = self.focused_monitor;
        if self.monitors.get(idx).is_some() && self.monitors[idx].active_ws_mut().add(hwnd) {
            let w = Window::from_raw(hwnd);
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
            self.focused_window = Some(hwnd);
            // In monocle mode the newest window becomes the
            // monocle target so it fills the work area.
            if self.monitors[idx].active_ws().monocle() {
                self.monitors[idx]
                    .active_ws_mut()
                    .set_monocle_window(Some(hwnd));
            }
            self.apply_layout_on(idx);
            self.focus_from_mouse = false;
            self.focus_and_update_border(hwnd);
        }
    }

    /// Moves a window that Windows has created but not shown yet into
    /// the slot it is going to occupy.
    ///
    /// Windows maps and paints a new window wherever it likes, often
    /// on the monitor the application was last used on, and only tells
    /// us afterwards. Moving it here, while it is still invisible,
    /// means the very first frame the user sees is already in place.
    ///
    /// Nothing is adopted. The workspace, the focus and the borders
    /// are all left untouched, and the window is only really taken
    /// over when its show event arrives and `add_and_focus` runs as
    /// usual. That matters because most invisible windows never become
    /// anything: applications routinely create hidden helper windows
    /// that carry a caption and would otherwise look adoptable. Moving
    /// one of those is harmless, whereas putting it in the layout
    /// would reflow every real window around a window that is never
    /// going to appear.
    ///
    /// The target rect is computed by asking the layout what it would
    /// produce with this window included. The handle is appended and
    /// removed again around that pure calculation, and no positions
    /// from it are applied to anything else, so the workspace is left
    /// exactly as it was.
    pub(super) fn pre_place(&mut self, hwnd: usize) {
        let idx = self.focused_monitor;
        let Some(state) = self.monitors.get(idx) else {
            return;
        };

        let rect = if state.active_ws().monocle() {
            // Monocle positions only its target and ignores the layout,
            // so asking compute_positions would return a slot that
            // nothing ever moves the window into. A window adopted onto
            // a monocle workspace becomes the monocle target itself, and
            // the target always fills the work area, so the destination
            // is known without adding the window to anything.
            self.monocle_rect(idx)
        } else {
            // Ask the layout where this window would go by adding it,
            // computing, and taking it straight back out again.
            if !self.monitors[idx].active_ws_mut().add(hwnd) {
                return;
            }
            let positions = self.compute_positions(idx);
            self.monitors[idx].active_ws_mut().remove(hwnd);

            let Some((_, rect)) = positions.into_iter().find(|(h, _)| *h == hwnd) else {
                return;
            };
            rect
        };
        let w = Window::from_raw(hwnd);
        // Move asynchronously. These windows have been created but not
        // shown, and some of them belong to threads that never pump
        // messages, so a synchronous SetWindowPos would never return and
        // would take the whole daemon down with it.
        if let Err(e) = w.move_async(&rect) {
            mosaico_core::log_debug!("~pre 0x{:X} move failed: {}", hwnd, e);
            return;
        }
        mosaico_core::log_debug!(
            "~pre 0x{:X} [{}] moved to mon {} ({},{} {}x{}) before it was shown",
            hwnd,
            w.class().unwrap_or_default(),
            idx,
            rect.x,
            rect.y,
            rect.width,
            rect.height
        );
    }

    pub(super) fn close_focused(&mut self) {
        let Some(hwnd) = self.focused_window else {
            return;
        };
        // Send WM_CLOSE so the window can handle its own shutdown gracefully.
        use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_CLOSE};
        // SAFETY: PostMessageW sends WM_CLOSE to a valid HWND owned by this process's focused window.
        unsafe {
            let _ = PostMessageW(
                Some(windows::Win32::Foundation::HWND(hwnd as *mut _)),
                WM_CLOSE,
                windows::Win32::Foundation::WPARAM(0),
                windows::Win32::Foundation::LPARAM(0),
            );
        }
    }

    pub(super) fn minimize_focused(&mut self) {
        let Some(hwnd) = self.focused_window else {
            return;
        };
        // ShowWindow(SW_MINIMIZE) fires EVENT_SYSTEM_MINIMIZESTART which
        // the event loop translates to WindowEvent::Minimized. That event
        // removes the window from the active workspace and re-tiles.
        Window::from_raw(hwnd).minimize();
    }
}
