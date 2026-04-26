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

    /// Sets the focused window, brings it to the foreground, and
    /// refreshes the focus border.
    pub(super) fn focus_and_update_border(&mut self, hwnd: usize) {
        self.focused_window = Some(hwnd);
        self.focused_maximized = Window::from_raw(hwnd).is_maximized();
        Window::from_raw(hwnd).set_foreground();
        if self.mouse_follows_focus && !self.focus_from_mouse {
            Self::move_cursor_to_window(hwnd);
        }
        self.focus_from_mouse = false;
        self.update_border();
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
        // Snapshot the current active-workspace contents per monitor so
        // we can compute color/visibility decisions without re-borrowing
        // self while we render.
        let visible: Vec<(usize, usize, mosaico_core::Rect)> = self.collect_visible_tiles();

        let visible_set: std::collections::HashSet<usize> =
            visible.iter().map(|(h, _, _)| *h).collect();

        // Drop borders whose owning window is no longer visible. Drop
        // runs `DestroyWindow`, which atomically hides + cleans up.
        self.borders.retain(|hwnd, _| visible_set.contains(hwnd));

        let focused_window = self.focused_window;
        let focused_monitor = self.focused_monitor;
        let focused_is_maximized = focused_window
            .map(|h| Window::from_raw(h).is_maximized())
            .unwrap_or(false);
        let focused_monocle = self
            .monitors
            .get(focused_monitor)
            .is_some_and(|m| m.active_ws().monocle());

        let unfocused_enabled = self.border_config.colors.unfocused_enabled();
        let width = self.border_config.width;
        let radius = self.border_config.corner_style.border_radius();

        let focused_color = parse_color(&self.border_config.colors.focused);
        let monocle_color = parse_color(&self.border_config.colors.monocle);
        let unfocused_color = parse_color(&self.border_config.colors.unfocused);

        for (hwnd, mon_idx, rect) in visible {
            let window = Window::from_raw(hwnd);
            // Skip windows that are visually absent: minimized windows
            // and any window covered by a focused-maximized neighbor on
            // the same monitor.
            if window.is_minimized()
                || (focused_is_maximized
                    && Some(hwnd) != focused_window
                    && mon_idx == focused_monitor)
            {
                if let Some(border) = self.borders.get(&hwnd) {
                    border.hide();
                }
                continue;
            }

            let is_focused = Some(hwnd) == focused_window;

            // In monocle mode, only the monocle window on the focused
            // monitor renders. Other windows on the focused monitor are
            // visually covered; windows on other monitors keep their
            // normal borders.
            if focused_monocle && mon_idx == focused_monitor && !is_focused {
                if let Some(border) = self.borders.get(&hwnd) {
                    border.hide();
                }
                continue;
            }

            // Skip unfocused borders entirely when disabled by config.
            if !is_focused && !unfocused_enabled {
                if let Some(border) = self.borders.get(&hwnd) {
                    border.hide();
                }
                continue;
            }

            let color = if is_focused {
                if focused_monocle && mon_idx == focused_monitor {
                    monocle_color
                } else {
                    focused_color
                }
            } else {
                unfocused_color
            };

            // Maximized focused windows skip border rendering (the
            // border would be invisible against the work-area edges or
            // overflow off-screen).
            if is_focused && focused_is_maximized {
                if let Some(border) = self.borders.get(&hwnd) {
                    border.hide();
                }
                continue;
            }

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

            border.show(&rect, color, width, radius, window.hwnd());
        }
    }

    /// Returns `(hwnd, monitor_idx, rect)` for every window currently
    /// laid out on its monitor's active workspace.
    fn collect_visible_tiles(&self) -> Vec<(usize, usize, Rect)> {
        let mut out = Vec::new();
        for (mi, mon) in self.monitors.iter().enumerate() {
            for &hwnd in mon.active_ws().handles() {
                if let Ok(rect) = Window::from_raw(hwnd).rect() {
                    out.push((hwnd, mi, rect));
                }
            }
        }
        out
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
