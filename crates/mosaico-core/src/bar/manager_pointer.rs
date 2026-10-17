//! What the pointer does on the bars, turned into actions.

use std::time::Instant;

use crate::Action;
use crate::action::{Origin, workspace_after};
use crate::config::{BarConfig, BarScroll, WidgetKind};

use super::super::{BarId, BarSide, BarTarget, Button, PointerEvent, WidgetClicks, WidgetRef};
use super::BarManager;

impl BarManager {
    /// Takes what the pointer did on the bar `bar` at `now`, and returns
    /// the action it asks for, with the bar as its origin, if it asks for
    /// one.
    ///
    /// A click is a press and a release of a button on the same item. A
    /// left click on a workspace label goes to that workspace. Any other
    /// click runs what its button runs on the kind of widget clicked (see
    /// [`WidgetClicks`]), and a label is part of the workspaces widget.
    /// Turns of the wheel add up, with `scroll = "workspaces"`, until the
    /// wheel goes quiet, see [`take_wheel`](Self::take_wheel), and a press
    /// on the bar drops the ones waiting there.
    /// What the pointer did on a bar that is gone, because the bars were
    /// made again since, is dropped, and so is everything while
    /// `[mouse] enabled` is off. A click on the bar of a monitor the tiler
    /// had no state for as the bar was drawn runs nothing.
    pub fn pointer(
        &mut self,
        bar: BarId,
        event: PointerEvent,
        now: Instant,
    ) -> Option<(Action, Origin)> {
        if !self.config.mouse.enabled {
            return None;
        }
        let scrolls = self.config.mouse.scroll == BarScroll::Workspaces;
        let slot = self.bars.iter_mut().find(|slot| slot.id == bar)?;
        if let PointerEvent::Wheel { notches, .. } = event {
            if scrolls {
                if slot.wheel.due().is_none() {
                    slot.wheel_from = slot.state.active_workspace;
                }
                slot.wheel.turn(notches, now);
            }
            return None;
        }
        if let PointerEvent::Press { .. } = event {
            slot.wheel.clear();
        }
        let (button, target) = slot.press.take(event, &slot.regions)?;
        let action = match target {
            BarTarget::Workspace(index) if button == Button::Left => {
                Action::GoToWorkspace(u8::try_from(index + 1).ok()?)
            }
            target => clicked(&self.config, target, button)?,
        };
        slot.tiled.then_some((action, Origin::Bar(bar.monitor)))
    }

    /// When the wheel turns waiting on a bar are first due, if any are.
    /// The daemon takes them with [`take_wheel`](Self::take_wheel) then.
    pub fn wheel_due(&self) -> Option<Instant> {
        self.bars.iter().filter_map(|slot| slot.wheel.due()).min()
    }

    /// The workspace switches the wheel asks for at `now`, one for each
    /// bar where it has gone quiet: to the workspace as many steps from the
    /// one the bar shows as the turns add up to, the way
    /// `goto-workspace-next` and `goto-workspace-prev` step, with
    /// `[workspaces] wrap` as the bar's state says, on the bar's monitor.
    /// Turns that add up to no step, or to a step past either end without
    /// wrap, switch nothing. So do turns that began on a workspace the bar
    /// no longer shows, since its monitor was switched some other way
    /// meanwhile, which drops them.
    pub fn take_wheel(&mut self, now: Instant) -> Vec<(Action, Origin)> {
        let mut switches = Vec::new();
        for slot in &mut self.bars {
            let Some(steps) = slot.wheel.take(now) else {
                continue;
            };
            let state = &slot.state;
            let active = state.active_workspace;
            if active != slot.wheel_from {
                slot.wheel.clear();
                continue;
            }
            let target = workspace_after(active, state.workspace_count, steps, state.wrap);
            if target != active
                && let Ok(n) = u8::try_from(target + 1)
            {
                switches.push((Action::GoToWorkspace(n), Origin::Bar(slot.id.monitor)));
            }
        }
        switches
    }
}

/// What a click of `button` on `target` runs with `config`, other than a
/// left click on a workspace label: what the button runs on the kind of
/// widget clicked. A workspace label is part of the workspaces widget.
fn clicked(config: &BarConfig, target: BarTarget, button: Button) -> Option<Action> {
    let (kind, side) = match target {
        BarTarget::Workspace(_) => (WidgetKind::Workspaces, BarSide::Left),
        BarTarget::Widget(WidgetRef { side, index }) => {
            let widgets = match side {
                BarSide::Left => &config.left,
                BarSide::Right => &config.right,
            };
            (widgets.get(index)?.kind(), side)
        }
    };
    WidgetClicks::new(&config.mouse).action(kind, side, button)
}

#[cfg(test)]
#[path = "tests/pointer_support.rs"]
mod support;

#[cfg(test)]
#[path = "tests/pointer_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/pointer_stale_tests.rs"]
mod stale_tests;

#[cfg(test)]
#[path = "tests/wheel_manager_tests.rs"]
mod wheel_tests;

#[cfg(test)]
#[path = "tests/wheel_bars_tests.rs"]
mod wheel_bars_tests;

#[cfg(test)]
#[path = "tests/wheel_drop_tests.rs"]
mod wheel_drop_tests;

#[cfg(test)]
#[path = "tests/wheel_switch_tests.rs"]
mod wheel_switch_tests;

#[cfg(test)]
#[path = "tests/click_through_tests.rs"]
mod click_through_tests;

#[cfg(test)]
#[path = "tests/widget_click_tests.rs"]
mod widget_tests;
