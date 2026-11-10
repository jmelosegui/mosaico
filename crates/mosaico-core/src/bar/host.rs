//! The daemon's side of the bar contract: the one provider it runs, and
//! the order in which the provider is placed, asked how far it reaches
//! and shown the state.

use std::time::Instant;

use crate::action::Origin;
use crate::api;
use crate::config::{BarConfig, Theme};
use crate::tiler::{Display, Platform, TilingManager};
use crate::{Action, Result};

use super::{BarId, BarProvider, PointerEvent};

/// The bar provider the daemon runs, with what the daemon tells it beside
/// the tiler's state: the daemon's own facts and the theme.
///
/// Every change to the monitors or the bar goes through here, so the order
/// [`BarProvider`] asks for holds whichever provider runs: a provider is
/// placed before it is asked how far it reaches, the tiles move once per
/// change, and a provider placed again is shown the state again.
pub struct BarHost {
    provider: Box<dyn BarProvider>,
    daemon: api::Daemon,
    theme: api::Theme,
    /// Whether the bars were placed since they were last shown the state.
    placed: bool,
}

impl BarHost {
    /// Runs `provider` for a daemon described by `daemon`, in `theme`.
    /// `daemon`'s provider is set to the name `provider` goes by.
    pub fn new(provider: Box<dyn BarProvider>, daemon: api::Daemon, theme: Theme) -> Self {
        let daemon = api::Daemon {
            provider: provider.name().to_string(),
            ..daemon
        };
        Self {
            provider,
            daemon,
            theme: api::Theme::of(theme),
            placed: false,
        }
    }

    /// Places the bars on `manager`'s monitors, keeps the tiles clear of
    /// them and shows them the state.
    pub fn start<P: Platform>(&mut self, manager: &mut TilingManager<P>) {
        let places = manager.places();
        self.provider.place(&places);
        manager.set_reach(self.provider.reach(&places));
        self.placed = true;
        self.show(manager);
    }

    /// Shows the bars the state, when it changed since they were last
    /// shown it or they were placed again since.
    pub fn show<P: Platform>(&mut self, manager: &mut TilingManager<P>) {
        self.shown(manager);
    }

    /// Once a second: shows the bars the state if it changed, or else has
    /// them draw again what changes on its own, such as a clock, so they
    /// draw once either way.
    pub fn tick<P: Platform>(&mut self, manager: &mut TilingManager<P>) {
        if !self.shown(manager) {
            self.provider.refresh();
        }
    }

    /// Shows the state as [`show`](Self::show) does, and tells whether it
    /// did.
    fn shown<P: Platform>(&mut self, manager: &mut TilingManager<P>) -> bool {
        let changed = manager.bar_state(&self.daemon, &self.theme).is_some();
        let show = changed || self.placed;
        if show {
            self.provider.show(manager.last_bar_state());
            self.placed = false;
        }
        show
    }

    /// Takes in a display change to `displays`: the bars move to the
    /// monitors as they will be, then the tiler rebuilds them, keeping
    /// clear of the bars.
    pub fn display_changed<P: Platform>(
        &mut self,
        manager: &mut TilingManager<P>,
        displays: Vec<Display>,
    ) {
        let places = manager.places_for(&displays);
        self.provider.place(&places);
        let reach = self.provider.reach(&places);
        manager.handle_display_change(displays, reach);
        self.placed = true;
    }

    /// Reads the work areas again, moves the bars to them and keeps the
    /// tiles clear of them, as when the system's own bars change. See
    /// [`TilingManager::reset_work_areas`] for when the read fails.
    pub fn reset_work_areas<P: Platform>(&mut self, manager: &mut TilingManager<P>) -> Result<()> {
        let provider = &mut self.provider;
        let read = manager.reset_work_areas(|places| {
            provider.place(places);
            provider.reach(places)
        });
        self.placed = true;
        read
    }

    /// Takes in a new `bar.toml`, then reads the work areas again as
    /// [`reset_work_areas`](Self::reset_work_areas) does, since the bar
    /// may now reach further or less far.
    pub fn reload<P: Platform>(
        &mut self,
        manager: &mut TilingManager<P>,
        config: BarConfig,
    ) -> Result<()> {
        self.provider.reload(config);
        self.reset_work_areas(manager)
    }

    /// Has the bars drawn in `theme` from the next [`show`](Self::show).
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = api::Theme::of(theme);
    }

    /// Notes whether the hotkeys are paused, from the next
    /// [`show`](Self::show).
    pub fn set_paused(&mut self, paused: bool) {
        self.daemon.paused = paused;
    }

    /// Notes the newer version on offer, if any, from the next
    /// [`show`](Self::show). An empty notice counts as none.
    pub fn set_update(&mut self, update: &str) {
        self.daemon.update = Some(update.to_string()).filter(|text| !text.is_empty());
    }

    /// The action what the pointer did on bar `bar` at `now` asks for.
    pub fn pointer(
        &mut self,
        bar: BarId,
        event: PointerEvent,
        now: Instant,
    ) -> Option<(Action, Origin)> {
        self.provider.pointer(bar, event, now)
    }

    /// When the bars next have actions to hand over on their own.
    pub fn due(&self) -> Option<Instant> {
        self.provider.due()
    }

    /// The actions the bars hand over at `now`.
    pub fn take_due(&mut self, now: Instant) -> Vec<(Action, Origin)> {
        self.provider.take_due(now)
    }

    /// Takes the bars off screen.
    pub fn stop(&mut self) {
        self.provider.stop();
    }
}
