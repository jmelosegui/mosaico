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
            Input::Reach(_) => self.fresh,
            _ => false,
        };
        // The steps that go with the bars being made again.
        let seed = matches!(
            input,
            Input::Displays { .. } | Input::WorkAreas { .. } | Input::Reach(_)
        );
        self.fresh = false;
        let mut plan = Plan::new(desktop, now);
        // Every action, from a key, a command or a bar, and a display
        // change, from when it is noted, leave where a dragged window lands
        // to its end, and the tiler stopping ends the drag (`drag_end.rs`)
        // and leaves the tiles it laid out live where they are.
        if matches!(
            input,
            Input::Action(..) | Input::Raise(_) | Input::DisplayChanging | Input::Displays { .. }
        ) {
            self.interrupt_drag(&mut plan);
        } else if let Input::Shutdown = input {
            self.end_drag(&mut plan);
            self.leave_live();
        }
        match input {
            Input::Action(action, origin) => {
                plan.from_pointer = origin.from_pointer();
                self.handle_action(&mut plan, &action, origin);
            }
            Input::Raise(window) => self.raise(&mut plan, window),
            Input::Event(event) => self.handle_event(&mut plan, &event),
            Input::DisplayChanging => {}
            Input::Displays { monitors, reach } => {
                self.handle_display_change(&mut plan, monitors, &reach)
            }
            Input::WorkAreas { monitors, reach } => {
                self.reset_work_areas(&mut plan, &monitors, &reach)
            }
            Input::Reach(reach) => self.reserve_bar_if_moved(&mut plan, &reach),
            Input::Config(config) => self.reload_config(&mut plan, &config),
            Input::Rules(rules) => self.reload_rules(&mut plan, rules),
            Input::BarProcesses(processes) => self.set_bar_processes(&mut plan, processes),
            Input::RefreshBorders => self.update_border(&mut plan),
            Input::RepaintAll => self.repaint_visible(&mut plan),
            Input::Shutdown => self.restore_all_windows(&mut plan),
        }
        self.end_drag_of_gone(&mut plan);
        // Whatever the step laid out, a monitor a drag no longer lays out
        // live goes back to its layout (`drag_live.rs`).
        self.settle_live(&mut plan);
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
