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
    /// shutdown so nothing is left hidden. Windows still moving are placed
    /// in their slots first, behind the frames on their way to them, and
    /// have no border drawn there: the tiler hides them all.
    pub fn restore_all_windows(&mut self) {
        let now = self.platform.now();
        self.applier.finish_motions(&mut self.platform, now);
        self.step(Input::Shutdown);
    }

    /// Applies a new layout, border, mouse and animation config, then
    /// retiles. A window already moving keeps its duration and easing
    /// while its target stays; one the retile sends elsewhere turns with
    /// the new ones, and with transitions off it snaps there. A session
    /// that has no frames keeps them off.
    pub fn reload_config(&mut self, config: &Config) {
        self.applier.set_animations(&config.animations);
        let mut config = config.clone();
        config.animations.enabled &= self.applier.has_frames();
        self.step(Input::Config(Box::new(config)));
    }

    /// Replaces the window rules and lets go of windows they no longer
    /// manage.
    pub fn reload_rules(&mut self, rules: Vec<WindowRule>) {
        self.step(Input::Rules(rules));
    }

    /// Stops every motion where its window is, as soon as a display change
    /// is noted, and has every placement snap until the layout is rebuilt
    /// for it, once it settles
    /// ([`handle_display_change`](Self::handle_display_change)). The
    /// windows it stops are placed at their targets after the rebuild, or
    /// when there is none ([`display_change_skipped`](Self::display_change_skipped)),
    /// unless something placed them or took them out of view meanwhile.
    pub fn display_changing(&mut self) {
        let now = self.platform.now();
        self.applier.display_changing(now);
    }

    /// Ends a display change the daemon does not rebuild for, as when it
    /// cannot read the monitors: the windows it stopped are placed at their
    /// targets, and placements may animate again.
    pub fn display_change_skipped(&mut self) {
        let now = self.platform.now();
        self.applier.display_changed(&mut self.platform, now);
    }

    /// Rebuilds the monitor state after a display configuration change.
    /// The rebuild places the windows it lays out at once, which ends the
    /// motions of the windows still moving, in their new slots. Then the
    /// windows the change stopped that nothing placed are placed at their
    /// targets.
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
        let now = self.platform.now();
        self.applier.display_changed(&mut self.platform, now);
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
