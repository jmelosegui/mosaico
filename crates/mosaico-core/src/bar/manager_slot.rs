//! One bar on screen, and what the manager keeps of it.

use crate::config::bar::BarConfig;

use super::super::{BarContent, BarId, BarState, BarSurface, ItemRegion, Press, WheelSteps};

/// A bar on screen, and what the manager keeps of it for the pointer.
pub(super) struct Slot {
    /// Who the bar is.
    pub(super) id: BarId,
    surface: Box<dyn BarSurface>,
    /// Where the bar drew its items last. Empty after a failed draw and
    /// after a hide, when there is nothing to click.
    pub(super) regions: Vec<ItemRegion>,
    /// The state the bar showed last, which the wheel steps from.
    pub(super) state: BarState,
    /// The press a click on the bar starts with.
    pub(super) press: Press,
    /// The turns of the wheel waiting to be applied.
    pub(super) wheel: WheelSteps,
    /// The workspace the bar showed as the turns waiting began, which
    /// they step from.
    pub(super) wheel_from: usize,
    /// Whether the bar lets the pointer through, as it does when made.
    click_through: bool,
}

impl Slot {
    /// The bar `surface`, just made as `id`.
    pub(super) fn new(id: BarId, surface: Box<dyn BarSurface>) -> Self {
        Self {
            id,
            surface,
            regions: Vec::new(),
            state: BarState::default(),
            press: Press::default(),
            wheel: WheelSteps::default(),
            wheel_from: 0,
            click_through: true,
        }
    }

    /// Has the bar let the pointer through, or take it, unless it does
    /// already. One that cannot is asked again on the next refresh.
    pub(super) fn let_through(&mut self, click_through: bool) {
        if click_through == self.click_through {
            return;
        }
        match self.surface.set_click_through(click_through) {
            Ok(()) => self.click_through = click_through,
            Err(e) => crate::log_debug!(
                "Bar on monitor {} not switched to click-through {click_through}: {e}",
                self.id.monitor
            ),
        }
    }

    /// Draws `content` in the style of `config`, for a monitor in `state`,
    /// and keeps where the items went.
    pub(super) fn draw(&mut self, config: &BarConfig, content: &BarContent, state: &BarState) {
        self.state = state.clone();
        match self.surface.draw(config, content) {
            Ok(regions) => self.regions = regions,
            Err(e) => {
                self.regions.clear();
                crate::log_debug!("Bar on monitor {} not drawn: {e}", self.id.monitor);
            }
        }
    }

    /// Takes the bar off screen, with nothing left on it to click.
    pub(super) fn hide(&mut self) {
        self.regions.clear();
        if let Err(e) = self.surface.hide() {
            crate::log_debug!("Bar on monitor {} not hidden: {e}", self.id.monitor);
        }
    }
}
