//! mosaico's own bar, as a [`BarProvider`].

use std::rc::Rc;
use std::time::Instant;

use crate::Action;
use crate::action::Origin;
use crate::api::State;
use crate::config::{BarConfig, Theme, ThemeConfig};
use crate::tiler::{MonitorPlace, Reach};

use super::{
    BarId, BarManager, BarProvider, BarRenderer, BarState, PointerEvent, PointerSink, SystemInfo,
};

/// The bar mosaico draws itself, through a [`BarRenderer`], on the
/// monitors `bar.toml` puts one on.
///
/// It reads mosaico only through the bar API's [`State`], as any other
/// bar does, colors included: the theme it draws in is the one the state
/// names.
pub struct BuiltinBar {
    bars: BarManager,
    theme: Theme,
    /// The monitors the bars were made for.
    places: Vec<MonitorPlace>,
    /// What each monitor's bar shows, in monitor order.
    states: Vec<BarState>,
    /// When what the bars show next changes on its own.
    redraw_at: Option<Instant>,
}

impl BuiltinBar {
    /// A bar with `config`, drawn by `renderer` in `theme`, reading the
    /// machine through `system` and reporting what the pointer does on it
    /// to `sink`. It has no bar on screen until it is placed.
    pub fn new(
        renderer: Box<dyn BarRenderer>,
        system: Box<dyn SystemInfo>,
        sink: Rc<dyn PointerSink>,
        config: BarConfig,
        theme: Theme,
    ) -> Self {
        Self {
            bars: BarManager::new(renderer, system, sink, config, Vec::new(), theme),
            theme,
            places: Vec::new(),
            states: Vec::new(),
            redraw_at: None,
        }
    }

    /// Draws every bar with what it shows at `now`, and notes when that
    /// next changes on its own.
    fn draw(&mut self, now: Instant) {
        self.bars.update(&self.states);
        self.redraw_at = self.bars.refresh_in().map(|wait| now + wait);
    }

    /// Takes the theme `state` names, when it names one and it differs
    /// from the one the bars are drawn in.
    fn follow_theme(&mut self, state: &State) {
        if state.theme.name.is_empty() {
            return;
        }
        let named = ThemeConfig {
            name: state.theme.name.clone(),
            flavor: state.theme.flavor.clone(),
        };
        let theme = named.resolve();
        if theme != self.theme {
            self.theme = theme;
            self.bars.resolve_colors(theme);
        }
    }
}

impl BarProvider for BuiltinBar {
    fn name(&self) -> &str {
        "mosaico"
    }

    /// Makes the bars again when a monitor came or went, or moved,
    /// changed size, work area or scale; leaves them be otherwise.
    fn place(&mut self, places: &[MonitorPlace]) {
        if self.places == places {
            return;
        }
        self.places = places.to_vec();
        let areas = places.iter().map(|place| place.work_area).collect();
        self.bars.rebuild_for_monitors(areas, self.theme);
    }

    fn reach(&self, places: &[MonitorPlace]) -> Reach {
        let indices = self.bars.bar_monitor_indices();
        Reach::bar_on(places, self.bars.bar_height(), indices)
    }

    fn show(&mut self, state: &State, now: Instant) {
        self.follow_theme(state);
        self.states = (state.monitors.iter())
            .filter_map(|monitor| BarState::from_state(state, &monitor.id))
            .collect();
        self.draw(now);
    }

    fn pointer(
        &mut self,
        bar: BarId,
        event: PointerEvent,
        now: Instant,
    ) -> Option<(Action, Origin)> {
        self.bars.pointer(bar, event, now)
    }

    /// The soonest of when the wheel's turns are due and when what the
    /// bars show changes on its own: the clock, or a reading of the
    /// machine.
    fn due(&self) -> Option<Instant> {
        self.bars
            .wheel_due()
            .into_iter()
            .chain(self.redraw_at)
            .min()
    }

    /// Draws the bars again from the state they hold, without it being
    /// built again, when what they show changed on its own, and hands over
    /// the wheel's turns that are due.
    fn wake(&mut self, now: Instant) -> Vec<(Action, Origin)> {
        if self.redraw_at.is_some_and(|at| now >= at) {
            self.draw(now);
        }
        self.bars.take_wheel(now)
    }

    /// Makes the bars again with `config`, in the same theme, and draws
    /// them with what they showed.
    fn reload(&mut self, config: BarConfig) {
        self.bars.reload(config);
        self.bars.resolve_colors(self.theme);
        self.bars.update(&self.states);
    }

    fn stop(&mut self) {
        self.bars.hide_all();
    }
}

#[cfg(test)]
#[path = "tests/builtin_tests.rs"]
mod tests;
