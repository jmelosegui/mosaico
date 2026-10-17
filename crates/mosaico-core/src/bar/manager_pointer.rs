//! What the pointer does on the bars, turned into actions.

use std::time::Instant;

use crate::Action;
use crate::action::{Origin, workspace_after};
use crate::config::BarScroll;

use super::super::{BarId, BarTarget, Button, PointerEvent};
use super::BarManager;

impl BarManager {
    /// Takes what the pointer did on the bar `bar` at `now`, and returns
    /// the action it asks for, with the bar as its origin, if it asks for
    /// one.
    ///
    /// A left click on a workspace label, a press and a release on it,
    /// goes to that workspace. Turns of the wheel add up, with
    /// `scroll = "workspaces"`, until the wheel goes quiet, see
    /// [`take_wheel`](Self::take_wheel), and a press on the bar drops the
    /// ones waiting there. Nothing else does anything yet.
    /// What the pointer did on a bar that is gone, because the bars were
    /// made again since, is dropped, and so is everything while
    /// `[mouse] enabled` is off.
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
        match slot.press.take(event, &slot.regions)? {
            (Button::Left, BarTarget::Workspace(index)) => {
                let n = u8::try_from(index + 1).ok()?;
                Some((Action::GoToWorkspace(n), Origin::Bar(bar.monitor)))
            }
            _ => None,
        }
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
