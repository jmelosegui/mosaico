//! The input the daemon sends the tiler besides actions and window
//! events: borders, repaints, shutdown, reloads and monitor changes.

use crate::Result;
use crate::config::{Config, WindowRule};

use super::super::{BarReservation, Display, Input, Platform};
use super::TilingManager;

impl<P: Platform> TilingManager<P> {
    /// Redraws every border to match the current window positions.
    pub fn refresh_border(&mut self) {
        self.step(Input::RefreshBorders);
    }

    /// Repaints every visible window, including those already in place.
    pub fn refresh_visible_windows(&mut self) {
        self.step(Input::RepaintAll);
    }

    /// Shows every window again and restores their frames. Called on
    /// shutdown so nothing is left hidden.
    pub fn restore_all_windows(&mut self) {
        self.step(Input::Shutdown);
    }

    /// Applies a new layout, border and mouse config, then retiles.
    pub fn reload_config(&mut self, config: &Config) {
        self.step(Input::Config(Box::new(config.clone())));
    }

    /// Replaces the window rules and lets go of windows they no longer
    /// manage.
    pub fn reload_rules(&mut self, rules: Vec<WindowRule>) {
        self.step(Input::Rules(rules));
    }

    /// Rebuilds the monitor state after a display configuration change.
    ///
    /// `new_monitors` are left to right, as
    /// [`Enumerator::monitors`](crate::tiler::Enumerator::monitors)
    /// lists them.
    pub fn handle_display_change(
        &mut self,
        new_monitors: Vec<Display>,
        bar_height: i32,
        bar_monitor_indices: &[usize],
    ) {
        self.step(Input::Displays {
            monitors: new_monitors,
            bar: bar(bar_height, bar_monitor_indices),
        });
    }

    /// Reserves the bar height at the top of each bar monitor, then
    /// retiles. An empty `bar_monitors` means all monitors.
    pub fn adjust_work_areas_for_bar(&mut self, bar_height: i32, bar_monitors: &[usize]) {
        self.step(Input::Bar(bar(bar_height, bar_monitors)));
    }

    /// Re-reads the OS work areas, then re-reserves the bar and retiles.
    ///
    /// When the monitors cannot be read, it still re-reserves the bar and
    /// retiles with the work areas it last saw, and returns why.
    pub fn reset_and_adjust_work_areas(
        &mut self,
        bar_height: i32,
        bar_monitors: &[usize],
    ) -> Result<()> {
        // No monitors leaves every work area as it was.
        let (monitors, read) = match self.platform.enumerator().monitors() {
            Ok(monitors) => (monitors, Ok(())),
            Err(e) => (Vec::new(), Err(e)),
        };
        self.step(Input::WorkAreas {
            monitors,
            bar: bar(bar_height, bar_monitors),
        });
        read
    }
}

fn bar(height: i32, monitors: &[usize]) -> BarReservation {
    BarReservation {
        height,
        monitors: monitors.to_vec(),
    }
}
