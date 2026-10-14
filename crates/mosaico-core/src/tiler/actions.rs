//! Actions the user triggers, from a hotkey or the CLI.

use crate::Action;

use super::{Effect, Plan, Tiler};

impl Tiler {
    /// Executes a user-triggered action.
    pub(super) fn handle_action(&mut self, plan: &mut Plan, action: &Action) {
        match action {
            Action::Focus(dir) => self.focus_direction(plan, *dir),
            Action::Move(dir) => self.move_direction(plan, *dir),
            Action::Retile => {
                // The user's repair for a stale desktop, which includes a
                // border that lost what the platform drew of it.
                plan.emit(Effect::RedrawBorders);
                self.retile_all(plan);
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
            Action::SendToWorkspace(n) => self.send_to_workspace(plan, *n),
            // Pausing is handled by the daemon before reaching here.
            Action::TogglePause => {}
        }
    }
}
