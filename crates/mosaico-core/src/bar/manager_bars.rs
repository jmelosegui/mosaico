//! Making the bars, making them again for new monitors or a new config,
//! and their colors and visibility.

use std::rc::Rc;

use crate::Rect;
use crate::config::Theme;
use crate::config::bar::BarConfig;

use super::super::{BarId, CpuTracker, WidgetClicks};
use super::BarManager;
use super::slot::Slot;

impl BarManager {
    /// Re-resolves the colors against a theme that may have changed.
    ///
    /// The whole config starts again from the copy loaded from the file,
    /// then every color, the `[colors]` table and each widget's `color`,
    /// is resolved against `theme`: a color given by name follows the new
    /// theme, and an empty one takes its default. Anything else changed in
    /// the config since it was loaded is undone too; only
    /// [`reload`](Self::reload) replaces that copy.
    pub fn resolve_colors(&mut self, theme: Theme) {
        self.config = self.raw_config.clone();
        self.config.resolve_colors(theme);
    }

    /// Hides every bar.
    pub fn hide_all(&mut self) {
        for bar in &mut self.bars {
            bar.hide();
        }
    }

    /// Replaces the bars after a display change: drops the old ones,
    /// re-resolves the colors as [`resolve_colors`](Self::resolve_colors)
    /// does and makes new ones for `monitor_areas`.
    pub fn rebuild_for_monitors(&mut self, monitor_areas: Vec<Rect>, theme: Theme) {
        self.monitor_areas = monitor_areas;
        self.resolve_colors(theme);
        self.create_bars();
    }

    /// Replaces the bars with ones for `config` and returns the new bar
    /// height. `config` also becomes the copy that
    /// [`resolve_colors`](Self::resolve_colors) starts from. The colors are
    /// left as loaded, so resolve them next. Each `[[mouse.click]]` entry
    /// the bars leave out is logged with why.
    pub fn reload(&mut self, config: BarConfig) -> i32 {
        report_clicks(&config);
        self.raw_config = config.clone();
        self.config = config;
        self.create_bars();
        self.cpu = self.cpu_tracker();
        self.bar_height()
    }

    /// Drops every bar, then makes one on each monitor the config shows
    /// one on, all of a new generation. What the pointer did on the bars
    /// dropped, a press or wheel turns waiting, goes with them.
    pub(super) fn create_bars(&mut self) {
        self.bars.clear();
        self.bar_monitor_indices.clear();
        self.generation += 1;
        if !self.config.enabled {
            return;
        }
        for (i, &area) in self.monitor_areas.iter().enumerate() {
            if !self.config.should_show_on(i) {
                continue;
            }
            let id = BarId {
                monitor: i,
                generation: self.generation,
            };
            match self.renderer.create(id, area, Rc::clone(&self.sink)) {
                Ok(surface) => {
                    self.bars.push(Slot::new(id, surface));
                    self.bar_monitor_indices.push(i);
                }
                // Same index as the `monitors` list in bar.toml.
                Err(e) => crate::log_warn!("No bar on monitor {i}: {e}"),
            }
        }
    }

    /// A CPU tracker while a CPU widget is configured.
    pub(super) fn cpu_tracker(&self) -> Option<CpuTracker> {
        self.config
            .has_cpu_widget()
            .then(|| CpuTracker::new(self.system.as_ref()))
    }
}

/// Logs each `[[mouse.click]]` entry of `config` the bars leave out, and
/// why, since nothing else tells the user that a click does not run it.
pub(super) fn report_clicks(config: &BarConfig) {
    for rejected in WidgetClicks::rejected(&config.mouse) {
        crate::log_warn!("bar.toml: {rejected}");
    }
}
