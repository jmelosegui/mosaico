//! The daemon's side of the bar contract: the one provider it runs, and
//! the order in which the provider is placed, asked how far it reaches
//! and shown the state.

use std::time::Instant;

use crate::action::Origin;
use crate::api;
use crate::config::{BarChoice, BarConfig, BarReserve, BarSection, Theme};
use crate::tiler::{Display, MonitorPlace, Platform, Reach, TilingManager};
use crate::{Action, Result};

use super::handover::{HANDOVER, Handover, reach_of};
use super::{ApiBar, BarId, BarProvider, NoBar, PointerEvent};

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
        }
    }

    /// Places the bars on `manager`'s monitors, keeps the tiles clear of
    /// them and shows them the state.
    pub fn start<P: Platform>(&mut self, manager: &mut TilingManager<P>) {
        let places = manager.places();
        self.provider.place(&places);
        self.reach = reach_of(self.provider.as_ref(), &mut self.handover, &places);
        manager.set_reach(self.reach.clone());
        self.placed = true;
        self.show(manager);
    }

    /// Shows the bars the state, when it changed since they were last
    /// shown it or they were placed again since.
    pub fn show<P: Platform>(&mut self, manager: &mut TilingManager<P>) {
        self.shown(manager);
    }

    /// Once a second, at `now`: lets go of the previous bar's space once
    /// its hold is over, then shows the bars the state if it changed, or
    /// else has them draw again what changes on its own, such as a clock,
    /// so they draw once either way.
    pub fn tick<P: Platform>(&mut self, manager: &mut TilingManager<P>, now: Instant) {
        if self.handover.as_ref().is_some_and(|held| held.over(now)) {
            self.handover = None;
            self.keep_clear(manager, &manager.places());
        }
        if !self.shown(manager) {
            self.provider.refresh();
        }
    }

    /// Shows the state as [`show`](Self::show) does, and tells whether it
    /// did. Nothing is built while the provider wants no state; it is
    /// shown the state once it does.
    fn shown<P: Platform>(&mut self, manager: &mut TilingManager<P>) -> bool {
        if !self.provider.wants_state() {
            return false;
        }
        let changed = manager.bar_state(&self.daemon, &self.api_theme).is_some();
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
        self.reach = reach_of(self.provider.as_ref(), &mut self.handover, &places);
        manager.handle_display_change(displays, self.reach.clone());
        self.placed = true;
    }

    /// Reads the work areas again, moves the bars to them and keeps the
    /// tiles clear of them, as when the system's own bars change. See
    /// [`TilingManager::reset_work_areas`] for when the read fails.
    pub fn reset_work_areas<P: Platform>(&mut self, manager: &mut TilingManager<P>) -> Result<()> {
        let provider = &mut self.provider;
        let handover = &mut self.handover;
        let mut reached = None;
        let read = manager.reset_work_areas(|places| {
            provider.place(places);
            let reach = reach_of(provider.as_ref(), handover, places);
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
        self.provider.place(&places);
        let until = now + HANDOVER;
        self.handover = (self.provider.connects_later())
            .then(|| Handover::begin(&self.reach, &self.provider.reach(&places), until))
            .flatten();
        self.keep_clear(manager, &places);
        self.placed = true;
        self.show(manager);
        true
    }

    /// Keeps the tiles clear of how far the bars reach into `places` now,
    /// moving them only when that changed.
    fn keep_clear<P: Platform>(&mut self, manager: &mut TilingManager<P>, places: &[MonitorPlace]) {
        let reach = reach_of(self.provider.as_ref(), &mut self.handover, places);
        if reach != self.reach {
            self.reach = reach;
            manager.set_reach(self.reach.clone());
        }
    }
}

/// The provider `choice` names, `make_builtin` making mosaico's own,
/// told `reserve`.
fn provider_for(
    choice: BarChoice,
    make_builtin: &mut MakeBuiltin,
    theme: Theme,
    reserve: &BarReserve,
) -> Box<dyn BarProvider> {
    let mut provider: Box<dyn BarProvider> = match choice {
        BarChoice::Mosaico => make_builtin(theme),
        BarChoice::External => Box::new(ApiBar::default()),
        BarChoice::Off => Box::new(NoBar),
    };
    provider.set_reserve(reserve);
    provider
}
