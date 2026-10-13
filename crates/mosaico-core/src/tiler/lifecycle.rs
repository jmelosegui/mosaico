//! Shutdown and configuration reload.

use crate::config::{Config, WindowRule};

use crate::WindowId;

use super::{Effect, Plan, Tiler};

impl Tiler {
    /// Shows every window on every workspace and gives each its normal
    /// frame back, so nothing is left hidden when the tiler stops.
    /// Borders are hidden first so they do not linger over windows.
    pub(super) fn restore_all_windows(&self, plan: &mut Plan) {
        self.hide_borders(plan);
        for window in self.all_windows() {
            plan.emit(Effect::Undecorate { window });
            plan.emit(Effect::Reveal { window });
        }
    }

    /// Applies a new configuration, then retiles.
    ///
    /// Per-workspace layouts are left alone: after startup they belong
    /// to the user's `cycle-layout` choices, which a reload must not
    /// undo. Changes to `[workspaces.layouts]` apply on restart.
    pub(super) fn reload_config(&mut self, plan: &mut Plan, config: &Config) {
        self.layout.gap = config.layout.gap;
        self.layout.ratio = config.layout.ratio;
        self.layout.hiding = config.layout.hiding;
        self.workspace_mode = config.workspaces.mode;
        self.border_config = config.borders.clone();
        self.mouse_follows_focus = config.mouse.follows_focus;
        for window in self.all_windows() {
            self.decorate(plan, window);
        }
        self.retile_all(plan);
    }

    /// Replaces the window rules and lets go of windows the new rules
    /// no longer manage.
    pub(super) fn reload_rules(&mut self, plan: &mut Plan, rules: Vec<WindowRule>) {
        self.rules = rules;
        self.adopt_rejected.clear();

        let mut removals = Vec::new();
        for (mi, mon) in self.monitors.iter().enumerate() {
            for (wi, ws) in mon.workspaces.iter().enumerate() {
                for &window in ws.windows() {
                    if !self.is_tileable(plan, window) {
                        removals.push((mi, wi, window));
                    }
                }
            }
        }

        let mut affected: Vec<usize> = Vec::new();
        for (mi, wi, window) in removals {
            plan.emit(Effect::Undecorate { window });
            self.monitors[mi].workspaces[wi].remove(window);
            crate::log_info!("-rule {window} (unmanaged by new rules)");
            if !affected.contains(&mi) {
                affected.push(mi);
            }
        }
        for idx in affected {
            self.apply_layout_on(plan, idx);
        }
    }

    /// Every managed window, on every workspace of every monitor.
    fn all_windows(&self) -> Vec<WindowId> {
        self.monitors
            .iter()
            .flat_map(|m| &m.workspaces)
            .flat_map(|ws| ws.windows().iter().copied())
            .collect()
    }
}
