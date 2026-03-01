//! Lifecycle and configuration reload for the tiling manager.

use mosaico_core::BspLayout;
use mosaico_core::config::WindowRule;

use crate::frame;
use crate::window::Window;

use super::TilingManager;

impl TilingManager {
    /// Shows all windows across every workspace and monitor.
    ///
    /// Called on daemon shutdown so that windows hidden by workspace
    /// switching are restored and not left invisible.
    pub fn restore_all_windows(&mut self) {
        self.hide_border();
        for mon in &self.monitors {
            for ws in &mon.workspaces {
                for &hwnd in ws.handles() {
                    let w = Window::from_raw(hwnd);
                    frame::reset_corner_preference(w.hwnd());
                    w.uncloak();
                    w.force_show();
                }
            }
        }
        self.hidden_by_switch.clear();
    }

    /// Hides a window using the configured strategy.
    pub(super) fn hide_window(&self, hwnd: usize) {
        let win = Window::from_raw(hwnd);
        match self.hiding {
            mosaico_core::config::HidingBehaviour::Cloak => win.cloak(),
            mosaico_core::config::HidingBehaviour::Hide => win.hide(),
            mosaico_core::config::HidingBehaviour::Minimize => win.minimize(),
        }
    }

    /// Shows a window, reversing the configured hiding strategy.
    pub(super) fn show_window(&self, hwnd: usize) {
        let win = Window::from_raw(hwnd);
        match self.hiding {
            mosaico_core::config::HidingBehaviour::Cloak => win.uncloak(),
            mosaico_core::config::HidingBehaviour::Hide => win.show(),
            mosaico_core::config::HidingBehaviour::Minimize => win.show(),
        }
    }

    /// Applies a new layout and border config, then retiles all windows.
    pub fn reload_config(&mut self, config: &mosaico_core::config::Config) {
        self.layout = BspLayout {
            gap: config.layout.gap,
            ratio: config.layout.ratio,
        };
        self.hiding = config.layout.hiding;
        self.border_config = config.borders.clone();
        self.apply_corner_preference_all();
        self.retile_all();
        self.update_border();
    }

    /// Replaces the window rules and removes windows that should no
    /// longer be managed under the new rule set.
    pub fn reload_rules(&mut self, rules: Vec<WindowRule>) {
        self.rules = rules;
        self.remove_newly_unmanaged();
    }

    /// Removes tiled windows that no longer pass `is_tileable` and
    /// retiles affected monitors.
    fn remove_newly_unmanaged(&mut self) {
        // Collect first (immutable) to avoid borrow conflicts.
        let mut removals: Vec<(usize, usize, usize)> = Vec::new();
        for (mi, mon) in self.monitors.iter().enumerate() {
            for (wi, ws) in mon.workspaces.iter().enumerate() {
                for &hwnd in ws.handles() {
                    if !self.is_tileable(hwnd) {
                        removals.push((mi, wi, hwnd));
                    }
                }
            }
        }
        if removals.is_empty() {
            return;
        }
        let mut affected: Vec<usize> = Vec::new();
        for &(mi, wi, hwnd) in &removals {
            frame::reset_corner_preference(Window::from_raw(hwnd).hwnd());
            self.monitors[mi].workspaces[wi].remove(hwnd);
            mosaico_core::log_info!("-rule 0x{hwnd:X} (unmanaged by new rules)");
            if !affected.contains(&mi) {
                affected.push(mi);
            }
        }
        for idx in affected {
            self.apply_layout_on(idx);
        }
    }

    /// Applies the current corner preference to every managed window.
    pub(super) fn apply_corner_preference_all(&self) {
        let style = self.border_config.corner_style;
        for mon in &self.monitors {
            for ws in &mon.workspaces {
                for &hwnd in ws.handles() {
                    frame::set_corner_preference(Window::from_raw(hwnd).hwnd(), style);
                }
            }
        }
    }
}
