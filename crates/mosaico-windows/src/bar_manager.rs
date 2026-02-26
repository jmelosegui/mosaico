//! Manages status bar instances across all monitors.
//!
//! Creates one [`Bar`] per monitor, handles updates, and provides
//! the 1-second timer tick for refreshing system widgets.

use mosaico_core::Rect;
use mosaico_core::config::Theme;
use mosaico_core::config::bar::{BarColors, BarConfig};

use crate::bar::widgets::cpu::CpuTracker;
use crate::bar::{Bar, BarState};

/// Manages bar overlay windows across all monitors.
pub struct BarManager {
    bars: Vec<Bar>,
    config: BarConfig,
    /// Colors as loaded from the file, before theme resolution.
    /// Kept so that switching themes can re-resolve from the originals
    /// instead of treating previously-resolved hex values as overrides.
    raw_colors: BarColors,
    /// Monitor work areas (original, before bar offset).
    monitor_rects: Vec<Rect>,
    /// Which monitor indices actually have a bar displayed.
    bar_monitor_indices: Vec<usize>,
    /// Stateful CPU tracker — only allocated when the CPU widget is
    /// configured, so it consumes zero resources when hidden.
    cpu_tracker: Option<CpuTracker>,
}

impl BarManager {
    /// Creates bars for selected monitors. Silently skips monitors
    /// where window creation fails.
    pub fn new(config: BarConfig, monitor_rects: Vec<Rect>, theme: Theme) -> Self {
        let raw_colors = config.colors.clone();
        let mut config = config;
        config.resolve_colors(theme);

        let (bars, bar_monitor_indices) = Self::create_bars(&config, &monitor_rects);

        let cpu_tracker = if config.has_cpu_widget() {
            Some(CpuTracker::new())
        } else {
            None
        };

        Self {
            bars,
            config,
            raw_colors,
            monitor_rects,
            bar_monitor_indices,
            cpu_tracker,
        }
    }

    /// Creates bars only for monitors that match the config filter.
    fn create_bars(config: &BarConfig, rects: &[Rect]) -> (Vec<Bar>, Vec<usize>) {
        let mut bars = Vec::new();
        let mut indices = Vec::new();
        if !config.enabled {
            return (bars, indices);
        }
        for (i, rect) in rects.iter().enumerate() {
            if config.should_show_on(i)
                && let Ok(bar) = Bar::new(*rect)
            {
                bars.push(bar);
                indices.push(i);
            }
        }
        (bars, indices)
    }

    /// Returns the bar height if the bar is enabled, 0 otherwise.
    pub fn bar_height(&self) -> i32 {
        if self.config.enabled {
            self.config.height
        } else {
            0
        }
    }

    /// Renders and shows all bars with the given per-monitor states.
    ///
    /// Only samples CPU when the CPU widget is configured.
    pub fn update(&mut self, states: &[BarState]) {
        let cpu = self.cpu_tracker.as_mut().map_or(0, CpuTracker::sample);
        for (i, bar) in self.bars.iter().enumerate() {
            let mon_idx = self.bar_monitor_indices[i];
            let mut state = states.get(mon_idx).cloned().unwrap_or_default();
            state.cpu_usage = cpu;
            bar.update(&self.config, &state);
        }
    }

    /// Returns which monitor indices have a bar displayed.
    pub fn bar_monitor_indices(&self) -> &[usize] {
        &self.bar_monitor_indices
    }

    /// Re-resolves bar colors from a (possibly changed) global theme.
    ///
    /// Resets to the raw (file-loaded) colors first so that theme
    /// defaults are re-applied rather than treated as user overrides.
    pub fn resolve_colors(&mut self, theme: Theme) {
        self.config.colors = self.raw_colors.clone();
        self.config.resolve_colors(theme);
    }

    /// Hides all bars.
    pub fn hide_all(&self) {
        for bar in &self.bars {
            bar.hide();
        }
    }

    /// Rebuilds bars for a new set of monitor rects (after display change).
    ///
    /// Drops existing bars, stores new monitor rects, re-resolves colors,
    /// and recreates bars.
    pub fn rebuild_for_monitors(&mut self, monitor_rects: Vec<Rect>, theme: Theme) {
        self.bars.clear();
        self.monitor_rects = monitor_rects;
        self.config.colors = self.raw_colors.clone();
        self.config.resolve_colors(theme);

        let (bars, indices) = Self::create_bars(&self.config, &self.monitor_rects);
        self.bars = bars;
        self.bar_monitor_indices = indices;
    }

    /// Recreates bars with a new config. Returns the new bar height.
    pub fn reload(&mut self, config: BarConfig) -> i32 {
        self.bars.clear();
        self.raw_colors = config.colors.clone();
        self.config = config;

        let (bars, indices) = Self::create_bars(&self.config, &self.monitor_rects);
        self.bars = bars;
        self.bar_monitor_indices = indices;

        // Create or drop the CPU tracker based on the new config.
        self.cpu_tracker = if self.config.has_cpu_widget() {
            Some(CpuTracker::new())
        } else {
            None
        };

        self.bar_height()
    }
}
