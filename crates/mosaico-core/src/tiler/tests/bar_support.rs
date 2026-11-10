//! What the bars show in tests: the bar API's state the tiler or the
//! manager builds, read back by each monitor's bar.

use crate::api;
use crate::bar::BarState;
use crate::tiler::{Desktop, Platform, Tiler, TilingManager};

use super::Harness;

/// The bar API's state `tiler` makes on `desktop`, with `daemon`. The
/// focused window has no title, which needs the manager.
pub(in crate::tiler) fn state_of(
    tiler: &Tiler,
    desktop: &dyn Desktop,
    daemon: &api::Daemon,
) -> api::State {
    let window = (tiler.focused_window()).map(|id| api::FocusedWindow {
        id,
        title: String::new(),
        program: String::new(),
    });
    api::State {
        daemon: tiler.api_daemon(daemon),
        theme: api::Theme::default(),
        monitors: tiler.api_monitors(&tiler.maximized_monitors(desktop)),
        focus: tiler.api_focus(window),
    }
}

/// What each monitor's bar shows of `state`, in monitor order.
pub(in crate::tiler) fn bars_of(state: &api::State) -> Vec<BarState> {
    (state.monitors.iter())
        .filter_map(|monitor| BarState::from_state(state, &monitor.id))
        .collect()
}

/// What each monitor's bar shows that `manager` makes, with `daemon`:
/// what it showed last when nothing changed.
pub(in crate::tiler) fn manager_bars<P: Platform>(
    manager: &mut TilingManager<P>,
    daemon: &api::Daemon,
) -> Vec<BarState> {
    let theme = api::Theme::default();
    let state = (manager.bar_state(daemon, &theme).cloned())
        .unwrap_or_else(|| manager.last_bar_state().clone());
    bars_of(&state)
}

impl Harness {
    /// What each monitor's bar shows, in monitor order.
    pub(in crate::tiler) fn bars(&self) -> Vec<BarState> {
        self.bars_with(&api::Daemon::default())
    }

    /// What each monitor's bar shows with `daemon`, in monitor order.
    pub(in crate::tiler) fn bars_with(&self, daemon: &api::Daemon) -> Vec<BarState> {
        bars_of(&state_of(&self.tiler, &self.desktop, daemon))
    }
}
