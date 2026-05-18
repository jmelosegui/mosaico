//! Focus management and per-window border overlays for the tiling manager.

use mosaico_core::Rect;
use mosaico_core::config::BorderColors;
use mosaico_core::window::Window as WindowTrait;
use windows::Win32::UI::WindowsAndMessaging::SetCursorPos;

use crate::border::{Border, Color};
use crate::window::Window;

use super::TilingManager;

/// Fallback border color used when a configured value fails to parse
/// as a hex string. Matches the historical default focused color.
const FALLBACK_COLOR: Color = Color {
    r: 0,
    g: 0xB4,
    b: 0xD8,
};

impl TilingManager {
    /// Moves the cursor to the center of the given window.
    fn move_cursor_to_window(hwnd: usize) {
        let Ok(rect) = Window::from_raw(hwnd).rect() else {
            return;
        };
        let cx = rect.x + rect.width / 2;
        let cy = rect.y + rect.height / 2;
        // SAFETY: SetCursorPos is safe to call with screen coordinates.
        unsafe {
            let _ = SetCursorPos(cx, cy);
        }
    }

    /// Sets the focused window and refreshes the focus border
    /// immediately, but defers the cross-process `SetForegroundWindow`
    /// call until the end of the current daemon batch.
    ///
    /// When several focus actions are processed in quick succession,
    /// only the final target ends up reaching the OS, so the user
    /// sees the border (and bar icon, which reads `focused_window`)
    /// race across windows at full keyboard speed while the slow
    /// foreground change happens once.
    pub(super) fn focus_and_update_border(&mut self, hwnd: usize) {
        self.focused_window = Some(hwnd);
        self.focused_maximized = Window::from_raw(hwnd).is_maximized();
        self.pending_empty_spawn = None;
        if let Some(anchor) = &self.focus_anchor {
            anchor.hide();
        }
        self.pending_foreground = Some((hwnd, self.focus_from_mouse));
        self.focus_from_mouse = false;
        self.update_border();
    }

    /// Returns true if a deferred `SetForegroundWindow` is waiting
    /// to be flushed.
    pub fn has_pending_foreground(&self) -> bool {
        self.pending_foreground.is_some()
    }

    /// Marks a monitor as needing its layout re-applied. The actual
    /// `SetWindowPos` storm is run by `flush_pending_retile` after
    /// the daemon's coalescing quiet window.
    pub(super) fn mark_retile(&mut self, monitor_idx: usize) {
        self.pending_retile.insert(monitor_idx);
    }

    /// Returns true if at least one monitor is waiting to be retiled.
    pub fn has_pending_retile(&self) -> bool {
        !self.pending_retile.is_empty()
    }

    /// Applies the deferred layout on every monitor marked dirty by
    /// `mark_retile`. Called by the daemon loop together with
    /// `flush_pending_foreground`.
    ///
    /// All monitors are retiled before a single trailing
    /// `update_border` runs. On a cross-monitor move this ensures the
    /// moved window has reached its new physical position before any
    /// `update_border` pass would otherwise read its stale rect and
    /// draw the focus border at the old slot (now occupied by the
    /// window that filled the gap on the source monitor).
    pub fn flush_pending_retile(&mut self) {
        let monitors: Vec<usize> = self.pending_retile.drain().collect();
        if monitors.is_empty() {
            return;
        }
        for idx in monitors {
            self.apply_layout_positions(idx);
        }
        self.update_border();
    }

    /// Applies the deferred `SetForegroundWindow` (and cursor move,
    /// if `mouse_follows_focus` is on) for the last focus request in
    /// the current batch. Called by the daemon loop after each
    /// iteration of action + event processing.
    pub fn flush_pending_foreground(&mut self) {
        let Some((hwnd, from_mouse)) = self.pending_foreground.take() else {
            return;
        };
        self.record_focus_intent(hwnd);
        Window::from_raw(hwnd).set_foreground();
        if self.mouse_follows_focus && !from_mouse {
            Self::move_cursor_to_window(hwnd);
        }
    }

    /// Moves the cursor to the focused window if `mouse_follows_focus` is
    /// enabled.  Called after move/swap operations where the focused window
    /// changes position but the focus itself does not change.
    pub(super) fn move_cursor_to_focused(&self) {
        if self.mouse_follows_focus
            && let Some(hwnd) = self.focused_window
        {
            Self::move_cursor_to_window(hwnd);
        }
    }

    /// Records that we just asked Win32 to focus `hwnd`, so the
    /// `Focused` event handler can recognize stale OS echoes and
    /// discard them. Keeps at most 8 entries from the last 1.5s.
    pub(super) fn record_focus_intent(&mut self, hwnd: usize) {
        let now = std::time::Instant::now();
        let cutoff = now - std::time::Duration::from_millis(1500);
        self.focus_intents.retain(|(_, t)| *t >= cutoff);
        self.focus_intents.push_back((hwnd, now));
        while self.focus_intents.len() > 8 {
            self.focus_intents.pop_front();
        }
    }

    /// Anchors focus to the given monitor when its active workspace is
    /// empty.  Positions the cursor at the work-area center (if
    /// `mouse_follows_focus`), activates the hidden focus-holder window,
    /// and records the anchored monitor so subsequent transient focus
    /// events (e.g. from PowerToys Run closing) do not drag
    /// `focused_monitor` back to the previous monitor before a
    /// user-launched app can spawn.
    pub(super) fn anchor_focus_to_monitor(&mut self, idx: usize) {
        let Some(mon) = self.monitors.get(idx) else {
            return;
        };
        let work_area = mon.work_area;
        if let Some(anchor) = &self.focus_anchor {
            anchor.activate_on(&work_area);
        }
        if self.mouse_follows_focus {
            let cx = work_area.x + work_area.width / 2;
            let cy = work_area.y + work_area.height / 2;
            // SAFETY: SetCursorPos is safe to call with screen coordinates.
            unsafe {
                let _ = SetCursorPos(cx, cy);
            }
        }
        self.pending_empty_spawn = Some(idx);
    }

    /// Returns true if `hwnd` matches a previously issued
    /// `set_foreground` intent that is no longer the most recent —
    /// i.e. a stale Win32 focus echo we should ignore.
    pub(super) fn is_stale_focus_echo(&mut self, hwnd: usize) -> bool {
        let cutoff = std::time::Instant::now() - std::time::Duration::from_millis(1500);
        // The deque is push_back-ordered by time, so the front is the
        // oldest entry.  Skip the full `retain` scan whenever the
        // oldest is still within the staleness window — the common
        // case during rapid navigation.
        if self.focus_intents.front().is_some_and(|(_, t)| *t < cutoff) {
            self.focus_intents.retain(|(_, t)| *t >= cutoff);
        }
        let latest = self.focus_intents.back().map(|(h, _)| *h);
        if latest == Some(hwnd) {
            return false;
        }
        self.focus_intents.iter().any(|(h, _)| *h == hwnd)
    }

    /// Refreshes every border overlay to match the current layout and
    /// focus state.
    ///
    /// - Creates borders for newly tiled windows on any monitor's
    ///   active workspace.
    /// - Drops borders for windows that have left an active workspace.
    /// - Recolors the focused window with the focused (or monocle)
    ///   color, others with the unfocused color.
    /// - Hides borders that should not be visible (e.g. neighbors of a
    ///   maximized focused window, every non-monocle window in monocle
    ///   mode, all unfocused windows when unfocused borders are
    ///   disabled).
    pub(super) fn update_border(&mut self) {
        // Move the scratch buffers out so we can borrow `&self.monitors`
        // while we fill them.  The fields are left empty until we put
        // the (now-populated) buffers back at the end — preserving the
        // allocated capacity across calls so no allocation happens on
        // steady-state focus changes.
        let mut visible = std::mem::take(&mut self.scratch_visible);
        let mut visible_set = std::mem::take(&mut self.scratch_visible_set);
        visible.clear();
        visible_set.clear();

        for (mi, mon) in self.monitors.iter().enumerate() {
            for &hwnd in mon.active_ws().handles() {
                if let Ok(rect) = Window::from_raw(hwnd).rect() {
                    visible.push((hwnd, mi, rect));
                    visible_set.insert(hwnd);
                }
            }
        }

        // Drop borders whose owning window is no longer visible. Drop
        // runs `DestroyWindow`, which atomically hides + cleans up.
        self.borders.retain(|hwnd, _| visible_set.contains(hwnd));

        let width = self.border_config.width;
        let radius = self.border_config.corner_style.border_radius();

        for &(hwnd, mon_idx, rect) in &visible {
            let Some(color) = self.decide_border_color(hwnd, mon_idx) else {
                if let Some(border) = self.borders.get(&hwnd) {
                    border.hide();
                }
                continue;
            };

            let border = match self.borders.get(&hwnd) {
                Some(b) => b,
                None => match Border::new() {
                    Ok(b) => self.borders.entry(hwnd).or_insert(b),
                    Err(e) => {
                        mosaico_core::log_warn!("border.create failed for 0x{:X}: {}", hwnd, e);
                        continue;
                    }
                },
            };

            border.show(&rect, color, width, radius, Window::from_raw(hwnd).hwnd());
        }

        self.scratch_visible = visible;
        self.scratch_visible_set = visible_set;
    }

    /// Returns the color the border for `hwnd` on monitor `mon_idx`
    /// should render with, or `None` if the border should be hidden.
    ///
    /// Encapsulates the visibility/color rules so single-window border
    /// updates (e.g. inline after `set_rect`) match what
    /// `update_border` would have decided in a full pass.
    fn decide_border_color(&self, hwnd: usize, mon_idx: usize) -> Option<Color> {
        let window = Window::from_raw(hwnd);
        let focused_window = self.focused_window;
        let focused_monitor = self.focused_monitor;
        let is_focused = Some(hwnd) == focused_window;
        let focused_is_maximized = focused_window
            .map(|h| Window::from_raw(h).is_maximized())
            .unwrap_or(false);
        let focused_monocle = self
            .monitors
            .get(focused_monitor)
            .is_some_and(|m| m.active_ws().monocle());
        let unfocused_enabled = self.border_config.colors.unfocused_enabled();

        if window.is_minimized()
            || (focused_is_maximized && !is_focused && mon_idx == focused_monitor)
        {
            return None;
        }
        if focused_monocle && mon_idx == focused_monitor && !is_focused {
            return None;
        }
        if !is_focused && !unfocused_enabled {
            return None;
        }
        if is_focused && focused_is_maximized {
            return None;
        }

        Some(if is_focused {
            if focused_monocle && mon_idx == focused_monitor {
                parse_color(&self.border_config.colors.monocle)
            } else {
                parse_color(&self.border_config.colors.focused)
            }
        } else {
            parse_color(&self.border_config.colors.unfocused)
        })
    }

    /// Repositions a single existing border immediately after the
    /// window it tracks has been moved via `set_rect`.
    ///
    /// Called inside the `apply_layout_positions` loop so the border's
    /// `SetWindowPos` lands in the same DWM compositor frame as the
    /// window's `SetWindowPos`, avoiding the visible one-frame lag
    /// where the window has reached its new slot but the border still
    /// sits at the previous position.
    ///
    /// Does not create new borders; that remains the responsibility of
    /// `update_border`, which runs after the layout loop to handle
    /// creation, destruction, and color/visibility transitions for
    /// windows that did not move.
    pub(super) fn show_border_for(&self, hwnd: usize, rect: &Rect, mon_idx: usize) {
        let Some(border) = self.borders.get(&hwnd) else {
            return;
        };
        match self.decide_border_color(hwnd, mon_idx) {
            Some(color) => {
                let width = self.border_config.width;
                let radius = self.border_config.corner_style.border_radius();
                border.show(rect, color, width, radius, Window::from_raw(hwnd).hwnd());
            }
            None => border.hide(),
        }
    }

    /// Hides every border overlay without dropping them. Called on
    /// daemon pause and on shutdown so borders stop rendering before
    /// windows are restored to their original positions.
    pub(super) fn hide_border(&self) {
        for border in self.borders.values() {
            border.hide();
        }
    }

    /// Re-positions all border overlays to match the current window rects.
    ///
    /// Call after work areas change (e.g. bar adjustment at startup)
    /// so borders reflect the final window positions.
    pub fn refresh_border(&mut self) {
        self.update_border();
    }

    /// Focuses the first window on the primary monitor at startup.
    pub(super) fn focus_initial(&mut self) {
        if let Some(mon) = self.monitors.first()
            && let Some(&hwnd) = mon.active_ws().handles().first()
        {
            self.focus_and_update_border(hwnd);
        }
    }
}

fn parse_color(hex: &str) -> Color {
    if hex == BorderColors::NONE {
        return FALLBACK_COLOR;
    }
    Color::from_hex(hex).unwrap_or(FALLBACK_COLOR)
}
