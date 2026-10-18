//! The status bars on every monitor.

use std::rc::Rc;

use crate::Rect;
use crate::config::Theme;
use crate::config::bar::BarConfig;

use super::{
    BarContent, BarRenderer, BarState, CpuTracker, PointerSink, SystemInfo, SystemReadings,
};

#[path = "manager_bars.rs"]
mod bars;
#[path = "manager_pointer.rs"]
mod pointer;
#[path = "manager_slot.rs"]
mod slot;

/// The status bars on every monitor, and the space they take.
///
/// Decides which monitors get a bar, how tall it is and what each one
/// shows, has a [`BarRenderer`] put the bars on screen, and reads the
/// machine through [`SystemInfo`]. The daemon refreshes it after every
/// change it makes and once a second, and hands it what the pointer does
/// on the bars, which it turns into actions.
pub struct BarManager {
    /// The bars on screen. Declared before `renderer`, so they are
    /// dropped before the renderer that made them.
    bars: Vec<slot::Slot>,
    /// The monitor index of each bar in `bars`.
    bar_monitor_indices: Vec<usize>,
    renderer: Box<dyn BarRenderer>,
    system: Box<dyn SystemInfo>,
    /// Where every bar reports what the pointer does on it.
    sink: Rc<dyn PointerSink>,
    /// The generation of the bars on screen. Bars made again get the
    /// next one, so the ids of the bars they replace are never reused.
    generation: u64,
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
    /// colors resolved against `theme`, reporting what the pointer does on
    /// it to `sink`. A monitor whose bar cannot be made goes without one.
    pub fn new(
        renderer: Box<dyn BarRenderer>,
        system: Box<dyn SystemInfo>,
        sink: Rc<dyn PointerSink>,
        config: BarConfig,
        monitor_areas: Vec<Rect>,
        theme: Theme,
    ) -> Self {
        let mut manager = Self {
            bars: Vec::new(),
            bar_monitor_indices: Vec::new(),
            renderer,
            system,
            sink,
            generation: 0,
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
    /// [`BarState::default`]. Each bar lets the pointer through while
    /// `[mouse] enabled` is off, and while its state says a maximized or
    /// fullscreen window is under it; otherwise it takes the pointer.
    pub fn update(&mut self, states: &[BarState]) {
        if self.bars.is_empty() {
            return;
        }
        let readings = SystemReadings::read(self.system.as_ref(), &self.config, self.cpu.as_mut());
        let fallback = BarState::default();
        for bar in &mut self.bars {
            let state = states.get(bar.id.monitor).unwrap_or(&fallback);
            bar.let_through(!self.config.mouse.enabled || state.maximized);
            let content = BarContent::build(&self.config, state, &readings);
            bar.draw(&self.config, &content, state);
        }
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
