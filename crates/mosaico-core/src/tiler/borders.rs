//! Focus borders around tiled windows.
//!
//! The focused window's border is drawn in the focused (or monocle)
//! color; the rest in the unfocused color when that is enabled. A color
//! that does not parse is drawn in [`BorderColors::FALLBACK`]. Borders
//! are created as windows appear on a visible workspace and destroyed
//! when they leave it.

use std::collections::HashSet;

use crate::Rect;
use crate::config::{BorderColors, Color};

use crate::WindowId;

use super::{BorderStyle, Effect, Plan, Tiler};

impl Tiler {
    /// Brings every border in line with the layout and focus.
    ///
    /// Destroys borders of windows no longer on screen, and draws,
    /// recolors or hides the rest, for example hiding the neighbors of a
    /// maximized focused window and everything but the monocle window.
    pub(super) fn update_border(&mut self, plan: &mut Plan) {
        let visible = self.collect_visible_tiles(plan);
        let on_screen: HashSet<WindowId> = visible.iter().map(|&(w, _, _)| w).collect();

        let mut gone: Vec<WindowId> = self
            .bordered
            .iter()
            .copied()
            .filter(|w| !on_screen.contains(w))
            .collect();
        gone.sort_unstable();
        for window in gone {
            self.bordered.remove(&window);
            plan.emit(Effect::DropBorder { window });
        }

        for (window, mon_idx, rect) in visible {
            let style = self.border_style(plan, window, mon_idx, rect);
            if style.is_none() && !self.bordered.contains(&window) {
                continue;
            }
            self.bordered.insert(window);
            plan.emit(Effect::Border { window, style });
        }
    }

    /// Redraws the border of a window that was just placed, so the two
    /// land on screen together. Borders that do not exist yet are left
    /// to `update_border`.
    pub(super) fn show_border_for(
        &self,
        plan: &mut Plan,
        window: WindowId,
        rect: Rect,
        mon_idx: usize,
    ) {
        if !self.bordered.contains(&window) {
            return;
        }
        let style = self.border_style(plan, window, mon_idx, rect);
        plan.emit(Effect::Border { window, style });
    }

    /// Hides every border without destroying it.
    pub(super) fn hide_borders(&self, plan: &mut Plan) {
        let mut all: Vec<WindowId> = self.bordered.iter().copied().collect();
        all.sort_unstable();
        for window in all {
            plan.emit(Effect::Border {
                window,
                style: None,
            });
        }
    }

    /// How the border around `window` on monitor `mon_idx` is drawn, or
    /// `None` when it should be hidden.
    fn border_style(
        &self,
        plan: &Plan,
        window: WindowId,
        mon_idx: usize,
        rect: Rect,
    ) -> Option<BorderStyle> {
        let is_focused = self.focused_window == Some(window);
        let on_focused_monitor = mon_idx == self.focused_monitor;
        let focused_is_maximized = self.focused_window.is_some_and(|w| plan.is_maximized(w));
        let focused_monocle = self
            .monitors
            .get(self.focused_monitor)
            .is_some_and(|m| m.active_ws().monocle());
        let colors = &self.border_config.colors;

        if plan.is_minimized(window) || (focused_is_maximized && !is_focused && on_focused_monitor)
        {
            return None;
        }
        if focused_monocle && on_focused_monitor && !is_focused {
            return None;
        }
        if !is_focused && !colors.unfocused_enabled() {
            return None;
        }
        if is_focused && focused_is_maximized {
            return None;
        }

        let color = if !is_focused {
            &colors.unfocused
        } else if focused_monocle && on_focused_monitor {
            &colors.monocle
        } else {
            &colors.focused
        };
        Some(BorderStyle {
            rect,
            color: Color::from_hex(color).unwrap_or(BorderColors::FALLBACK),
            width: self.border_config.width,
            radius: self.border_config.corner_style.border_radius(),
        })
    }

    /// `(window, monitor, rect)` for every window laid out on its
    /// monitor's active workspace.
    fn collect_visible_tiles(&self, plan: &Plan) -> Vec<(WindowId, usize, Rect)> {
        let mut out = Vec::new();
        for (mi, mon) in self.monitors.iter().enumerate() {
            for &window in mon.active_ws().windows() {
                if let Some(rect) = plan.rect(window) {
                    out.push((window, mi, rect));
                }
            }
        }
        out
    }
}
