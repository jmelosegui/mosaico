//! Actions the user triggers, from a hotkey, the CLI or the pointer on a
//! bar.

use crate::Action;
use crate::action::{Origin, Scope, workspace_after};

use super::floating::Axis;
use super::{Effect, Plan, Tiler};

impl Tiler {
    /// Executes a user-triggered action that comes from `origin`.
    ///
    /// An action from a bar whose monitor the tiler has no state for does
    /// nothing. One that acts on a monitor acts on the bar's: when that
    /// monitor does not have focus, focus moves there first, as if the
    /// user had gone there, and the action then does what its key does.
    pub(super) fn handle_action(&mut self, plan: &mut Plan, action: &Action, origin: Origin) {
        if let Some(mon) = origin.monitor() {
            if mon >= self.monitors.len() {
                return;
            }
            if action.scope() == Scope::Monitor && mon != self.focused_monitor {
                self.enter_monitor(plan, mon);
            }
        }
        match action {
            Action::Focus(dir) => {
                if !self.focus_from_float(plan, *dir) {
                    self.focus_direction(plan, *dir);
                }
            }
            Action::Move(dir) => {
                if !self.nudge_float(plan, *dir) {
                    self.move_direction(plan, *dir);
                }
            }
            Action::Retile => {
                // The user's repair for a stale desktop, which includes a
                // border that lost what the platform drew of it, and the
                // way back to the configured ratio, or an equal share down
                // a stack, from the splits they moved.
                plan.emit(Effect::RedrawBorders);
                self.retile_from_ratio(plan);
            }
            Action::ToggleMonocle => self.toggle_monocle(plan),
            Action::CloseFocused => {
                if let Some(window) = self.focused_window {
                    plan.emit(Effect::Close { window });
                }
            }
            Action::MinimizeFocused => {
                // The minimize is reported back as a `Minimized` event,
                // which is what takes the window out of the layout.
                if let Some(window) = self.focused_window {
                    plan.emit(Effect::Minimize { window });
                }
            }
            Action::CycleLayout => self.cycle_layout(plan),
            Action::GoToWorkspace(n) => self.goto_workspace(plan, *n),
            Action::GoToWorkspaceNext => self.step_workspace(plan, 1),
            Action::GoToWorkspacePrev => self.step_workspace(plan, -1),
            Action::SendToWorkspace(n) => self.send_to_workspace(plan, *n),
            // Pausing is handled by the daemon before reaching here.
            Action::TogglePause => {}
            Action::ToggleFloat => self.toggle_float(plan),
            Action::CycleFloat => self.cycle_float(plan),
            Action::GrowWidth => self.resize_float(plan, Axis::Width, true),
            Action::ShrinkWidth => self.resize_float(plan, Axis::Width, false),
            Action::GrowHeight => self.resize_float(plan, Axis::Height, true),
            Action::ShrinkHeight => self.resize_float(plan, Axis::Height, false),
        }
    }

    /// Switches the focused monitor to the workspace `steps` steps from
    /// the one on screen, every monitor in global mode, the way
    /// [`workspace_after`] steps with `[workspaces] wrap`. A step that
    /// stays on the workspace on screen changes nothing.
    fn step_workspace(&mut self, plan: &mut Plan, steps: i32) {
        let Some(mon) = self.monitors.get(self.focused_monitor) else {
            return;
        };
        let count = mon.workspaces.len();
        let target = workspace_after(mon.active_workspace, count, steps, self.workspace_wrap);
        if let Ok(n) = u8::try_from(target + 1) {
            self.goto_workspace(plan, n);
        }
    }
}

#[cfg(test)]
#[path = "tests/bar_click_support.rs"]
mod support;

#[cfg(test)]
#[path = "tests/bar_click_tests.rs"]
mod bar_click_tests;

#[cfg(test)]
#[path = "tests/bar_click_focus_tests.rs"]
mod bar_click_focus_tests;

#[cfg(test)]
#[path = "tests/workspace_step_tests.rs"]
mod step_tests;

#[cfg(test)]
#[path = "tests/workspace_wrap_tests.rs"]
mod wrap_tests;

#[cfg(test)]
#[path = "tests/bar_widget_tests.rs"]
mod bar_widget_tests;
