//! The input the daemon sends the tiler besides actions and window
//! events: borders, repaints, shutdown, reloads and monitor changes.

use crate::Result;
use crate::config::{Config, WindowRule};

use super::super::{Display, Input, MonitorPlace, Platform, Reach};
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
    /// unless something placed them or took them out of view meanwhile. A
    /// drag going on is interrupted, so no drop is decided against the
    /// monitors as they were.
    pub fn display_changing(&mut self) {
        let now = self.platform.now();
        self.applier.display_changing(now);
        self.step(Input::DisplayChanging);
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
    /// lists them. `reach` names them by the ids
    /// [`places_for`](Self::places_for) gives them.
    pub fn handle_display_change(&mut self, new_monitors: Vec<Display>, reach: Reach) {
        self.step(Input::Displays {
            monitors: new_monitors,
            reach,
        });
        let now = self.platform.now();
        self.applier.display_changed(&mut self.platform, now);
    }

    /// Keeps tiles clear of how far the bar reaches into each monitor,
    /// then retiles.
    pub fn set_reach(&mut self, reach: Reach) {
        self.step(Input::Reach(reach));
    }

    /// Re-reads the OS work areas, then keeps tiles clear of what `bar`
    /// says it reaches and retiles, once.
    ///
    /// `bar` is given the monitors with the work areas just read, so a bar
    /// can move there and say how far it reaches from them. When the
    /// monitors cannot be read, `bar` is given them as last seen, the
    /// tiles are kept clear of its reach on those work areas, and the
    /// reason is returned.
    pub fn reset_work_areas(&mut self, bar: impl FnOnce(&[MonitorPlace]) -> Reach) -> Result<()> {
        // No monitors leaves every work area as it was.
        let (monitors, read) = match self.platform.enumerator().monitors() {
            Ok(monitors) => (monitors, Ok(())),
            Err(e) => (Vec::new(), Err(e)),
        };
        let places = if monitors.is_empty() {
            self.places()
        } else {
            self.places_for(&monitors)
        };
        let reach = bar(&places);
        self.step(Input::WorkAreas { monitors, reach });
        read
    }

    /// The connected monitors, left to right, as a bar needs to know them.
    pub fn places(&self) -> Vec<MonitorPlace> {
        self.tiler.places()
    }

    /// The monitors as [`places`](Self::places) will list them once
    /// `displays` are taken in by
    /// [`handle_display_change`](Self::handle_display_change).
    pub fn places_for(&self, displays: &[Display]) -> Vec<MonitorPlace> {
        self.tiler.places_for(displays)
    }
}
