//! Shutdown and configuration reload.

use crate::config::{Config, WindowRule};

use crate::WindowId;

use super::{Effect, Plan, Tiler};

impl Tiler {
    /// Shows every window on every workspace and gives each its normal
    /// frame back, so nothing is left hidden when the tiler stops.
    /// Borders are hidden first so they do not linger over windows.
    pub(super) fn restore_all_windows(&mut self, plan: &mut Plan) {
        self.hide_borders(plan);
        for window in self.all_windows() {
            self.undecorate(plan, window);
            plan.emit(Effect::Reveal { window });
        }
    }

    /// Applies a new configuration, then retiles.
    ///
    /// Per-workspace layouts are left alone: after startup they belong
    /// to the user's `cycle-layout` choices, which a reload must not
    /// undo. Changes to `[workspaces.layouts]` apply on restart. A
    /// hiding strategy the platform does not have is swapped for one it
    /// has, as at startup.
    pub(super) fn reload_config(&mut self, plan: &mut Plan, config: &Config) {
        self.layout.gap = config.layout.gap;
        self.layout.ratio = config.layout.ratio;
        // What the platform can do never changes, so an unchanged
        // strategy keeps the choice already made, and its warning is not
        // logged again on every edit of the config.
        if config.layout.hiding != self.configured_hiding {
            self.configured_hiding = config.layout.hiding;
            self.layout.hiding = self.hiding_support.choose(config.layout.hiding);
        }
        self.workspace_mode = config.workspaces.mode;
        self.border_config = config.borders.clone();
        self.mouse_follows_focus = config.mouse.follows_focus;
        for window in self.all_windows() {
            self.decorate(plan, window);
        }
        self.retile_all(plan);
    }

    /// Replaces the window rules and lets go of windows the new rules
    /// no longer manage. One hidden by a workspace switch is shown
    /// again as it is let go, so it is not left out of view.
    pub(super) fn reload_rules(&mut self, plan: &mut Plan, rules: Vec<WindowRule>) {
        self.rules = rules;
        self.adopt_rejected.clear();

        let mut removals = Vec::new();
        for (mi, mon) in self.monitors.iter().enumerate() {
            for (wi, ws) in mon.workspaces.iter().enumerate() {
                for &window in ws.windows() {
                    if !self.still_managed(plan, window) {
                        removals.push((mi, wi, window));
                    }
                }
            }
        }

        let mut affected: Vec<usize> = Vec::new();
        for (mi, wi, window) in removals {
            self.undecorate(plan, window);
            let ws = &mut self.monitors[mi].workspaces[wi];
            ws.remove(window);
            // As when a window leaves any other way, a workspace whose
            // monocle window goes is no longer in monocle.
            if ws.monocle() && ws.monocle_window() == Some(window) {
                ws.set_monocle(false);
                ws.set_monocle_window(None);
            }
            // Shows it the way it was hidden, if it was, then forgets it.
            self.hidden_how.release(plan, window);
            crate::log_info!("-rule {window} (unmanaged by new rules)");
            if !affected.contains(&mi) {
                affected.push(mi);
            }
        }
        for idx in affected {
            self.apply_layout_on(plan, idx);
        }
    }

    /// Whether the rules still manage a window the tiler manages.
    ///
    /// A window hidden by a workspace switch is out of view by the
    /// tiler's own doing, so only the rules judge it, on what the
    /// platform says about it whether or not it is in view (#111).
    fn still_managed(&self, plan: &Plan, window: WindowId) -> bool {
        if !self.hidden_how.contains(window) {
            return self.is_tileable(plan, window);
        }
        plan.facts(window)
            .is_some_and(|facts| self.rules_manage(&facts))
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
