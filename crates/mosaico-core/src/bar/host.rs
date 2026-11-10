//! The daemon's side of the bar contract: the one provider it runs, and
//! the order in which the provider is placed, asked how far it reaches
//! and shown the state.

use std::sync::Arc;
use std::time::Instant;

use crate::action::Origin;
use crate::api;
use crate::config::{BarChoice, BarConfig, BarReserve, BarSection, Theme};
use crate::tiler::{Display, Platform, Reach, TilingManager};
use crate::{Action, Result};

use super::handover::{HANDOVER, Handover, reach_of};
use super::provider::provider_for;
use super::reservations::Reservations;
use super::{BarId, BarProvider, PointerEvent};

/// Makes mosaico's own bar in a theme, with `bar.toml` read afresh.
pub type MakeBuiltin = Box<dyn FnMut(Theme) -> Box<dyn BarProvider>>;

/// The bar provider the daemon runs, with what the daemon tells it beside
/// the tiler's state: the daemon's own facts and the theme.
///
/// Every change to the monitors or the bar goes through here, so the order
/// [`BarProvider`] asks for holds whichever provider runs: a provider is
/// placed before it is asked how far it reaches, the tiles move once per
/// change, and a provider placed again is shown the state again.
pub struct BarHost {
    provider: Box<dyn BarProvider>,
    choice: BarChoice,
    make_builtin: MakeBuiltin,
    daemon: api::Daemon,
    theme: Theme,
    api_theme: api::Theme,
    /// Whether the bars were placed since they were last shown the state.
    placed: bool,
    /// How far the bars reached when the tiles last moved for them.
    reach: Reach,
    /// `[bar.reserve]`, which a bar that cannot say how far it reaches
    /// keeps.
    reserve: BarReserve,
    /// The space the previous bar kept, held after a switch until the new
    /// one says how far it reaches.
    handover: Option<Handover>,
    /// Where the bars that read the state through the bar API get it.
    hub: Option<Arc<api::Hub>>,
    /// The space bars reserve through the bar API.
    reservations: Reservations,
    /// The processes of those bars, as the tiler was last told them.
    processes: Vec<u32>,
}

impl BarHost {
    /// Runs the provider `section` picks for a daemon described by
    /// `daemon`, in `theme`. `make_builtin` is only called when mosaico's
    /// own bar is picked, now or later. `daemon`'s provider is set to the
    /// name the provider goes by.
    pub fn new(
        section: &BarSection,
        mut make_builtin: MakeBuiltin,
        daemon: api::Daemon,
        theme: Theme,
    ) -> Self {
        let choice = section.choice();
        let provider = provider_for(choice, &mut make_builtin, theme, &section.reserve);
        let daemon = api::Daemon {
            provider: provider.name().to_string(),
            ..daemon
        };
        Self {
            provider,
            choice,
            make_builtin,
            daemon,
            theme,
            api_theme: api::Theme::of(theme),
            placed: false,
            reach: Reach::default(),
            reserve: section.reserve.clone(),
            handover: None,
            hub: None,
            reservations: Reservations::default(),
            processes: Vec::new(),
        }
    }

    /// Places the bars on `manager`'s monitors, keeps the tiles clear of
    /// them and shows them the state.
    pub fn start<P: Platform>(&mut self, manager: &mut TilingManager<P>) {
        let places = manager.places();
        self.provider.place(&places);
        self.reach = reach_of(self.wanted(&places), &mut self.handover);
        manager.set_reach(self.reach.clone());
        self.placed = true;
        self.show(manager);
    }

    /// Shows the bars the state, when it changed since they were last
    /// shown it or they were placed again since.
    pub fn show<P: Platform>(&mut self, manager: &mut TilingManager<P>) {
        self.shown(manager);
    }

    /// Publishes the state to `hub` from the next [`show`](Self::show)
    /// on, while a bar API session follows it, whichever provider runs.
    pub fn publish_to(&mut self, hub: Arc<api::Hub>) {
        self.hub = Some(hub);
    }

    /// Shows the state as [`show`](Self::show) does, and publishes it to
    /// the hub when it changed or the hub's is not fresh. Nothing is built
    /// while neither the provider nor a session wants the state.
    fn shown<P: Platform>(&mut self, manager: &mut TilingManager<P>) {
        let hub = self.hub.as_ref().filter(|hub| hub.watched());
        let wanted = self.provider.wants_state();
        if !wanted && hub.is_none() {
            return;
        }
        let changed = manager.bar_state(&self.daemon, &self.api_theme).is_some();
        if wanted && (changed || self.placed) {
            let now = manager.platform().now();
            self.provider.show(manager.last_bar_state(), now);
            self.placed = false;
        }
        if let Some(hub) = hub.filter(|hub| changed || !hub.fresh()) {
            hub.publish(manager.last_bar_state());
        }
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
        self.reach = reach_of(self.wanted(&places), &mut self.handover);
        manager.handle_display_change(displays, self.reach.clone());
        self.placed = true;
    }

    /// Reads the work areas again, moves the bars to them and keeps the
    /// tiles clear of them, as when the system's own bars change. See
    /// [`TilingManager::reset_work_areas`] for when the read fails.
    pub fn reset_work_areas<P: Platform>(&mut self, manager: &mut TilingManager<P>) -> Result<()> {
        let provider = &mut self.provider;
        let handover = &mut self.handover;
        let reservations = &self.reservations;
        let mut reached = None;
        let read = manager.reset_work_areas(|places| {
            provider.place(places);
            let wanted = provider.reach(places).furthest(&reservations.reach(places));
            let reach = reach_of(wanted, handover);
            reached = Some(reach.clone());
            reach
        });
        if let Some(reach) = reached {
            self.reach = reach;
        }
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
        self.theme = theme;
        self.api_theme = api::Theme::of(theme);
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

    /// When the bars next need [`wake`](Self::wake): the provider's
    /// [`due`](BarProvider::due), the end of the hold on the previous
    /// bar's space, or of the space of a session that ended, whichever is
    /// sooner. `None` while nothing waits, so an idle daemon with no clock
    /// to draw is never woken for the bars.
    pub fn due(&self) -> Option<Instant> {
        let held = self.handover.as_ref().map(Handover::until);
        let reserved = self.reservations.due();
        (self.provider.due().into_iter())
            .chain(held)
            .chain(reserved)
            .min()
    }

    /// Does what is due at `now`: lets go of the previous bar's space once
    /// its hold is over, and of the space of a session that ended once its
    /// grace is, and has the provider draw again what changed on its own.
    /// Returns the actions the bars hand over, such as the wheel's.
    pub fn wake<P: Platform>(
        &mut self,
        manager: &mut TilingManager<P>,
        now: Instant,
    ) -> Vec<(Action, Origin)> {
        let over = self.handover.as_ref().is_some_and(|held| held.over(now));
        if over {
            self.handover = None;
        }
        if self.reservations.expire(now) || over {
            self.space_changed(manager);
            self.show(manager);
        }
        self.provider.wake(now)
    }

    /// Takes the bars off screen, and lets go of the space reserved.
    pub fn stop(&mut self) {
        self.provider.stop();
        self.reservations.clear();
    }

    /// The bar running, as `[bar] provider` picks it.
    pub fn choice(&self) -> BarChoice {
        self.choice
    }

    /// Runs the provider `section` picks with its `[bar.reserve]`, and
    /// tells whether that changed the bar. The bars running are taken off
    /// screen first, then the new ones are placed, the tiles keep clear of
    /// them, moving only where they reach further or less far, and they are
    /// shown the state.
    ///
    /// A bar that only says how far it reaches once it connects gets the
    /// space the previous bar kept for [`HANDOVER`] from `now`, or until it
    /// reaches into the monitor. A change to the reserve alone moves the
    /// tiles where the space kept changed, and is not a change of bar.
    pub fn choose<P: Platform>(
        &mut self,
        manager: &mut TilingManager<P>,
        section: &BarSection,
        now: Instant,
    ) -> bool {
        let choice = section.choice();
        if choice == self.choice && section.reserve == self.reserve {
            return false;
        }
        self.reserve = section.reserve.clone();
        let places = manager.places();
        if choice == self.choice {
            self.provider.set_reserve(&self.reserve);
            self.keep_clear(manager, &places);
            return false;
        }
        self.provider.stop();
        self.provider = provider_for(choice, &mut self.make_builtin, self.theme, &self.reserve);
        self.choice = choice;
        self.daemon.provider = self.provider.name().to_string();
        let provider = &self.provider;
        self.reservations.retain(|bar| provider.accepts(bar));
        self.provider.place(&places);
        let until = now + HANDOVER;
        self.handover = (self.provider.connects_later())
            .then(|| Handover::begin(&self.reach, &self.wanted(&places), until))
            .flatten();
        self.tell_processes(manager);
        self.keep_clear(manager, &places);
        self.placed = true;
        self.show(manager);
        true
    }
}

#[path = "host_space.rs"]
mod space;
