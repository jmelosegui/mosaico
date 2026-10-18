//! One step of the tiler: an input in, the effects that answer it out.

use std::time::Instant;

use super::plan::StepKind;
use super::{Desktop, Effect, Input, Plan, Tiler};

impl Tiler {
    /// Handles one input and returns the effects to apply, in order.
    pub fn step(&mut self, input: Input, desktop: &dyn Desktop, now: Instant) -> Vec<Effect> {
        // Monitors and work areas are in flux during a display change, and
        // while the bar takes its space right after the start.
        let rebuild = match input {
            Input::Displays { .. } => true,
            Input::Bar(_) => self.fresh,
            _ => false,
        };
        // The steps that go with the bars being made again.
        let seed = matches!(
            input,
            Input::Displays { .. } | Input::WorkAreas { .. } | Input::Bar(_)
        );
        self.fresh = false;
        let mut plan = Plan::new(desktop, now);
        match input {
            Input::Action(action, origin) => {
                plan.from_pointer = origin.from_pointer();
                self.handle_action(&mut plan, &action, origin);
            }
            Input::Event(event) => self.handle_event(&mut plan, &event),
            Input::Displays { monitors, bar } => {
                self.handle_display_change(&mut plan, monitors, &bar)
            }
            Input::WorkAreas { monitors, bar } => self.reset_work_areas(&mut plan, &monitors, &bar),
            Input::Bar(bar) => self.reserve_bar(&mut plan, &bar),
            Input::Config(config) => self.reload_config(&mut plan, &config),
            Input::Rules(rules) => self.reload_rules(&mut plan, rules),
            Input::RefreshBorders => self.update_border(&mut plan),
            Input::RepaintAll => self.repaint_visible(&mut plan),
            Input::Shutdown => self.restore_all_windows(&mut plan),
        }
        if seed {
            self.seed_filling(desktop);
        }
        plan.finish(&self.step_kind(rebuild))
    }

    /// What decides which placements of a step may animate, for a step
    /// that rebuilds the layout from scratch when `rebuild`: a display
    /// change, the tiler's start, or the bar reservation right after it.
    pub(super) fn step_kind(&self, rebuild: bool) -> StepKind {
        StepKind {
            enabled: self.animations.is_on(),
            rebuild,
        }
    }

    /// Turns transitions off in the step decision, whatever the config
    /// says, for a session with no frames to move windows on: every
    /// placement is marked to snap, and the log hears of none on its own.
    /// The manager keeps them off through reloads.
    pub(super) fn turn_off_transitions(&mut self) {
        self.animations.enabled = false;
    }
}

#[cfg(test)]
#[path = "tests/step_transition_tests.rs"]
mod tests;
