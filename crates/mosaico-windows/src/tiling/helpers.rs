//! Window lookup and management helpers for the tiling manager.

use mosaico_core::window::Window as WindowTrait;

use crate::frame;
use crate::monitor;
use crate::window::Window;

use super::TilingManager;

/// How long a pre placed window may stay in the layout without ever
/// becoming visible before it is swept out again.
const PRE_PLACED_TTL: std::time::Duration = std::time::Duration::from_secs(2);

/// What the periodic sweep should do with a window that was placed
/// before Windows showed it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PrePlacedAction {
    /// Still inside the TTL. Leave it alone and look again next tick.
    Keep,
    /// It is on screen and still passes the rules, so its slot is real
    /// even though the show event never completed the adoption. Stop
    /// tracking it and leave the layout alone.
    Settle,
    /// It never appeared, or it appeared and the rules reject it.
    /// Reclaim the slot it was holding.
    Reclaim,
}

/// Decides the fate of one pre placed window.
///
/// Deliberately free of Win32 calls so the policy can be tested on its
/// own. `tileable` is the caller's already resolved answer to "is this
/// window on screen and still something we manage".
pub(super) fn pre_placed_action(
    elapsed: std::time::Duration,
    ttl: std::time::Duration,
    tileable: bool,
) -> PrePlacedAction {
    if elapsed < ttl {
        PrePlacedAction::Keep
    } else if tileable {
        PrePlacedAction::Settle
    } else {
        PrePlacedAction::Reclaim
    }
}

/// Whether a window that Windows has created but not shown yet should
/// be placed into the layout now.
///
/// Monocle is excluded on purpose. `apply_layout_positions` positions
/// only the monocle window and returns, so a pre placed window would
/// take a slot the layout never moves and would still appear wherever
/// the OS put it. Leaving monocle on the existing path means the show
/// event adopts the window normally, exactly as it does today.
pub(super) fn should_pre_place(monocle: bool, tracked: bool, passes_rules: bool) -> bool {
    !monocle && !tracked && passes_rules
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

    /// Places a window that Windows has created but not shown yet.
    ///
    /// Windows maps and paints a new window wherever it likes, often
    /// on the monitor the application was last used on, and only tells
    /// us afterwards. Positioning the window here, while it is still
    /// invisible, means the very first frame the user sees is already
    /// in the tile it belongs to.
    ///
    /// Focus is deliberately left untouched: the window is not on
    /// screen yet, so pointing the border or the foreground at it
    /// would be wrong. That half of the adoption runs from
    /// `finish_pre_placed` once the show event arrives.
    pub(super) fn pre_place(&mut self, hwnd: usize) {
        let idx = self.focused_monitor;
        if self.monitors.get(idx).is_none() {
            return;
        }
        if !self.monitors[idx].active_ws_mut().add(hwnd) {
            return;
        }
        let w = Window::from_raw(hwnd);
        let class = w.class().unwrap_or_default();
        mosaico_core::log_info!(
            "+pre 0x{:X} [{}] to mon {} ws {} (now {})",
            hwnd,
            class,
            idx,
            self.monitors[idx].active_workspace + 1,
            self.monitors[idx].active_ws().len()
        );
        frame::set_corner_preference(w.hwnd(), self.border_config.corner_style);
        self.pre_placed.insert(hwnd, std::time::Instant::now());
        self.apply_layout_on(idx);
    }

    /// Completes the adoption of a pre placed window once it is
    /// visible, running the focus half that `pre_place` skipped.
    ///
    /// Does nothing for windows that were not pre placed, so the
    /// `Created` handler can call it unconditionally on the path where
    /// the window is already tracked.
    pub(super) fn finish_pre_placed(&mut self, hwnd: usize) {
        if self.pre_placed.remove(&hwnd).is_none() {
            return;
        }
        let Some(idx) = self.owning_monitor(hwnd) else {
            return;
        };
        // Focus the window before layout so monocle mode sizes the
        // correct window, exactly as add_and_focus does.
        self.focused_window = Some(hwnd);
        if self.monitors[idx].active_ws().monocle() {
            self.monitors[idx]
                .active_ws_mut()
                .set_monocle_window(Some(hwnd));
        }
        // The window may have resized itself between the create and
        // the show, so re-apply the layout before taking focus.
        self.apply_layout_on(idx);
        self.focus_from_mouse = false;
        self.focus_and_update_border(hwnd);
    }

    /// Drops pre placed windows that never became visible.
    ///
    /// A window can be created and then abandoned, or shown much later
    /// than we assumed. Without this sweep its slot would stay in the
    /// layout forever, leaving a gap where nothing is drawn. Entries
    /// whose window did show up but was never adopted (for instance a
    /// rule that only matches once the title is set) lose their slot
    /// too, so the rules stay authoritative.
    pub fn sweep_pre_placed(&mut self) {
        if self.pre_placed.is_empty() {
            return;
        }
        let now = std::time::Instant::now();
        let entries: Vec<(usize, std::time::Duration)> = self
            .pre_placed
            .iter()
            .map(|(&hwnd, &placed)| (hwnd, now.duration_since(placed)))
            .collect();

        let mut affected: Vec<usize> = Vec::new();
        for (hwnd, elapsed) in entries {
            match pre_placed_action(elapsed, PRE_PLACED_TTL, self.is_tileable(hwnd)) {
                PrePlacedAction::Keep => {}
                PrePlacedAction::Settle => {
                    self.pre_placed.remove(&hwnd);
                }
                PrePlacedAction::Reclaim => {
                    self.pre_placed.remove(&hwnd);
                    if let Some((mi, wi)) = self.find_window(hwnd) {
                        self.monitors[mi].workspaces[wi].remove(hwnd);
                        mosaico_core::log_info!("-pre 0x{:X} (never shown)", hwnd);
                        if !affected.contains(&mi) {
                            affected.push(mi);
                        }
                    }
                }
            }
        }
        for idx in affected {
            self.apply_layout_on(idx);
        }
    }

    /// Forgets a pre placed window without touching the layout.
    ///
    /// Used by the `Destroyed` handler, where the removal from the
    /// workspace is handled separately.
    pub(super) fn forget_pre_placed(&mut self, hwnd: usize) {
        self.pre_placed.remove(&hwnd);
    }

    /// Returns true if `hwnd` was placed before Windows showed it and
    /// is still waiting for its show event.
    pub(super) fn is_pre_placed(&self, hwnd: usize) -> bool {
        self.pre_placed.contains_key(&hwnd)
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
