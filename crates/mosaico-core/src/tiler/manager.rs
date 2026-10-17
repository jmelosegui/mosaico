//! The tiling manager: feeds the tiler its input and applies the
//! effects it returns.

use std::time::Instant;

use crate::action::Origin;
use crate::config::{Config, WindowRule};
use crate::{Action, HidingSupport, Result, WindowEvent};

use super::applier::Applier;
use super::{HiddenFile, Input, Platform, Settings, Tiler};

#[path = "manager_bars.rs"]
mod bars;
#[path = "manager_frames.rs"]
mod frames;
#[path = "manager_inputs.rs"]
mod inputs;

/// Manages tiled windows across all connected monitors.
///
/// Runs a [`Tiler`] on the desktop of the platform `P`: hands it the
/// daemon's input, applies the effects it returns through the platform,
/// holds back the ones that may wait while input keeps arriving, and
/// drops the events its own effects cause.
pub struct TilingManager<P> {
    tiler: Tiler,
    applier: Applier,
    platform: P,
}

impl<P: Platform> TilingManager<P> {
    /// Starts tiling the windows on the desktop of `platform` that
    /// `rules` accept, laid out, bordered and hidden the way `config`
    /// says.
    ///
    /// `hidden` is where the record of the windows hidden by workspace
    /// switching is kept, so that the next daemon can show them again if
    /// this one is killed. `None` keeps no record. Fails when the
    /// platform cannot list the monitors or the windows.
    pub fn new(
        mut platform: P,
        hidden: Option<HiddenFile>,
        config: &Config,
        rules: Vec<WindowRule>,
    ) -> Result<Self> {
        // Show what a killed or crashed daemon left hidden before looking
        // at the windows, so they are found and adopted like any other.
        let hiding = HidingSupport::new(|how| platform.control().hiding_events(how));
        let mut applier = Applier::new(hidden, hiding);
        applier.set_animations(&config.animations);
        let now = platform.now();
        applier.restore_previous(&mut platform, now);

        let displays = platform.enumerator().monitors()?;
        let windows = platform.enumerator().windows()?;
        let settings = Settings {
            layout: config.layout.clone(),
            workspaces: config.workspaces.clone(),
            borders: config.borders.clone(),
            rules,
            mouse_follows_focus: config.mouse.follows_focus,
            animations: config.animations.clone(),
            hiding_support: hiding,
        };
        let now = platform.now();
        let (tiler, effects) = Tiler::new(settings, &displays, &windows, platform.desktop(), now);
        let mut manager = Self {
            tiler,
            applier,
            platform,
        };
        manager.applier.apply(effects, &mut manager.platform, now);
        Ok(manager)
    }

    fn step(&mut self, input: Input) {
        let now = self.platform.now();
        // While transitions are off, the platform forgets the windows it no
        // longer needs to post to before the tiler reads the desktop.
        self.applier.settle_while_off(&self.platform);
        let desktop = self.applier.view(&self.platform, now);
        let effects = self.tiler.step(input, &desktop, now);
        self.applier.apply(effects, &mut self.platform, now);
    }

    /// Executes a user-triggered action from a hotkey or the CLI.
    pub fn handle_action(&mut self, action: &Action) {
        self.handle_action_from(action, Origin::Focused);
    }

    /// Executes a user-triggered action that comes from `origin`, such as
    /// a click on the bar of a monitor, which it then acts on.
    pub fn handle_action_from(&mut self, action: &Action, origin: Origin) {
        self.step(Input::Action(*action, origin));
    }

    /// Handles a window event, unless it is an echo of our own changes.
    pub fn handle_event(&mut self, event: &WindowEvent) {
        if let WindowEvent::Destroyed { window } = *event {
            self.applier.forget(window);
        }
        let now = self.platform.now();
        if let Some(reason) = self.applier.echo(event, &mut self.platform, now) {
            crate::log_debug!("{} suppressed ({})", event, reason);
            return;
        }
        self.step(Input::Event(event.clone()));
    }

    /// When the focus changes and placements held back, waiting for
    /// input to go quiet, started being held, or `None` when nothing is
    /// held.
    pub fn held_since(&self) -> Option<Instant> {
        self.applier.held_since()
    }

    /// Applies everything held back. The daemon calls this when the
    /// scheduler says it is due.
    pub fn flush_pending(&mut self) {
        let now = self.platform.now();
        self.applier.flush(&mut self.platform, now);
    }

    /// Returns the total number of managed windows.
    pub fn window_count(&self) -> usize {
        self.tiler.window_count()
    }

    /// The tiler's model of the desktop: the monitors, their workspaces
    /// and windows, and what has focus.
    pub fn tiler(&self) -> &Tiler {
        &self.tiler
    }

    /// The platform the manager runs on.
    pub fn platform(&self) -> &P {
        &self.platform
    }
}
