//! The status bars on every monitor.

use crate::Rect;
use crate::config::Theme;
use crate::config::bar::BarConfig;

use super::{
    BarContent, BarRenderer, BarState, BarSurface, CpuTracker, SystemInfo, SystemReadings,
};

/// The status bars on every monitor, and the space they take.
///
/// Decides which monitors get a bar, how tall it is and what each one
/// shows, has a [`BarRenderer`] put the bars on screen, and reads the
/// machine through [`SystemInfo`]. The daemon refreshes it after every
/// change it makes and once a second.
pub struct BarManager {
    /// The bars on screen. Declared before `renderer`, so they are
    /// dropped before the renderer that made them.
    bars: Vec<Box<dyn BarSurface>>,
    /// The monitor index of each bar in `bars`.
    bar_monitor_indices: Vec<usize>,
    renderer: Box<dyn BarRenderer>,
    system: Box<dyn SystemInfo>,
    /// The config, with its colors resolved against the theme.
    config: BarConfig,
    /// The config as loaded from the file, before theme resolution.
    /// Kept so that switching themes resolves every color again from what
    /// the file says, the `[colors]` table and each widget's `color`,
    /// instead of from the previous theme's hex values.
    raw_config: BarConfig,
    /// Monitor work areas, before the bar is reserved.
    monitor_areas: Vec<Rect>,
    /// Only kept while a CPU widget is configured, so the CPU is not
    /// read otherwise.
    cpu: Option<CpuTracker>,
}

impl BarManager {
    /// Makes a bar on each monitor the config shows one on, with the
    /// colors resolved against `theme`. A monitor whose bar cannot be
    /// made goes without one.
    pub fn new(
        renderer: Box<dyn BarRenderer>,
        system: Box<dyn SystemInfo>,
        config: BarConfig,
        monitor_areas: Vec<Rect>,
        theme: Theme,
    ) -> Self {
        let mut manager = Self {
            bars: Vec::new(),
            bar_monitor_indices: Vec::new(),
            renderer,
            system,
            raw_config: config.clone(),
            config,
            monitor_areas,
            cpu: None,
        };
        manager.config.resolve_colors(theme);
        manager.create_bars();
        manager.cpu = manager.cpu_tracker();
        manager
    }

    /// Returns the space the bars take at the top of their monitors: the
    /// bar height while a monitor has a bar, 0 otherwise, since the tiler
    /// reads the empty list of bar monitors as every monitor.
    pub fn bar_height(&self) -> i32 {
        if self.bars.is_empty() {
            0
        } else {
            self.config.height
        }
    }

    /// Returns which monitor indices have a bar displayed.
    pub fn bar_monitor_indices(&self) -> &[usize] {
        &self.bar_monitor_indices
    }

    /// Renders and shows every bar with its monitor's state.
    ///
    /// The machine is read once for all bars, and not at all while no
    /// monitor has a bar. A monitor that `states` has no entry for shows
    /// [`BarState::default`].
    pub fn update(&mut self, states: &[BarState]) {
        if self.bars.is_empty() {
            return;
        }
        let readings = SystemReadings::read(self.system.as_ref(), &self.config, self.cpu.as_mut());
        let fallback = BarState::default();
        for (bar, &monitor) in self.bars.iter_mut().zip(&self.bar_monitor_indices) {
            let state = states.get(monitor).unwrap_or(&fallback);
            let content = BarContent::build(&self.config, state, &readings);
            if let Err(e) = bar.draw(&self.config, &content) {
                crate::log_debug!("Bar on monitor {monitor} not drawn: {e}");
            }
        }
    }

    /// Re-resolves the colors against a theme that may have changed.
    ///
    /// Every color starts again from the file, the `[colors]` table and
    /// each widget's `color`, so a color given by name follows the new
    /// theme and an empty one takes the new theme's default, instead of
    /// keeping the old theme's hex value as if the user had set it.
    pub fn resolve_colors(&mut self, theme: Theme) {
        self.config = self.raw_config.clone();
        self.config.resolve_colors(theme);
    }

    /// Hides every bar.
    pub fn hide_all(&mut self) {
        for (bar, monitor) in self.bars.iter_mut().zip(&self.bar_monitor_indices) {
            if let Err(e) = bar.hide() {
                crate::log_debug!("Bar on monitor {monitor} not hidden: {e}");
            }
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
    /// height. The colors are left as loaded, so resolve them next.
    pub fn reload(&mut self, config: BarConfig) -> i32 {
        self.raw_config = config.clone();
        self.config = config;
        self.create_bars();
        self.cpu = self.cpu_tracker();
        self.bar_height()
    }

    /// Drops every bar, then makes one on each monitor the config shows
    /// one on.
    fn create_bars(&mut self) {
        self.bars.clear();
        self.bar_monitor_indices.clear();
        if !self.config.enabled {
            return;
        }
        for (i, &area) in self.monitor_areas.iter().enumerate() {
            if !self.config.should_show_on(i) {
                continue;
            }
            match self.renderer.create(area) {
                Ok(bar) => {
                    self.bars.push(bar);
                    self.bar_monitor_indices.push(i);
                }
                // Same index as the `monitors` list in bar.toml.
                Err(e) => crate::log_warn!("No bar on monitor {i}: {e}"),
            }
        }
    }

    /// A CPU tracker while a CPU widget is configured.
    fn cpu_tracker(&self) -> Option<CpuTracker> {
        self.config
            .has_cpu_widget()
            .then(|| CpuTracker::new(self.system.as_ref()))
    }
}

#[cfg(test)]
#[path = "tests/manager_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/theme_tests.rs"]
mod theme_tests;

#[cfg(test)]
#[path = "tests/reload_tests.rs"]
mod reload_tests;

#[cfg(test)]
#[path = "tests/manager_update_tests.rs"]
mod update_tests;

#[cfg(test)]
#[path = "tests/reservation_tests.rs"]
mod reservation_tests;
