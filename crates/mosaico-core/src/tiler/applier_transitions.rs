//! What the applier keeps for the placements that animate, and what it
//! tells the log of them.

use std::collections::HashMap;
use std::fmt;
use std::time::Instant;

use crate::WindowId;
use crate::config::AnimationConfig;
use crate::frames::FrameOutcome;
use crate::motion::{Ending, Journal, Landed, LandedLine, Motions, Placed};

use super::super::Applier;
use super::super::borders::MotionBorder;

/// What the applier keeps for the placements that animate.
#[derive(Debug)]
pub(in crate::tiler::applier) struct Transitions {
    /// The windows moving to their placements.
    pub(in crate::tiler::applier) motions: Motions,
    /// What the border of each moving window does until it is in place.
    pub(in crate::tiler::applier) borders: HashMap<WindowId, MotionBorder>,
    /// What the platform found of each window on the last frame, kept from
    /// one frame to the next.
    pub(in crate::tiler::applier) outcomes: Vec<FrameOutcome>,
    /// How placements animate, as configured.
    pub(in crate::tiler::applier) animations: AnimationConfig,
    /// Whether the platform can move windows through frames.
    pub(in crate::tiler::applier) frames_supported: bool,
    /// Whether the last frames failed to be posted, for a reason other than
    /// the platform not having frames.
    pub(in crate::tiler::applier) frames_failing: bool,
    /// What the log is told of the motions: a line per step and per
    /// motion, never one per frame.
    pub(in crate::tiler::applier) journal: Journal,
    /// Every line logged, for the tests to read.
    #[cfg(test)]
    pub(in crate::tiler::applier) logged: Vec<String>,
}

impl Default for Transitions {
    fn default() -> Self {
        Self {
            motions: Motions::default(),
            borders: HashMap::new(),
            outcomes: Vec::new(),
            animations: AnimationConfig::default(),
            frames_supported: true,
            frames_failing: false,
            journal: Journal::default(),
            #[cfg(test)]
            logged: Vec::new(),
        }
    }
}

impl Applier {
    /// Starts a new step: the windows the next apply or flush sets moving
    /// go together (see `Motions::begin_step`). The step before is logged
    /// first.
    pub(in crate::tiler::applier) fn begin_step(&mut self) {
        self.log_step();
        self.transitions.motions.begin_step();
        self.transitions.journal.begin_step();
    }

    /// Logs the windows the last step set moving, if it set any, before
    /// its first frame, the next step, or the end of any motion, whichever
    /// comes first; after the lines of the steps it closed as it turned
    /// the last of their motions (see `Journal::moved`).
    pub(in crate::tiler::applier) fn log_step(&mut self) {
        for line in self.transitions.journal.closed_steps() {
            self.log(line);
        }
        if let Some(line) = self.transitions.journal.step_line() {
            self.log(line);
        }
    }

    /// Logs how the motion of `window` ended at `now`, for `ending`, if the
    /// window was moving, with how the placement that ended it was made,
    /// `placed`, if one did, and has the log hear when the window lands, if
    /// it is still to (`applier_landing.rs`); and its step's line when it
    /// was the last of the step to end, after the line of the step that set
    /// it moving.
    pub(in crate::tiler::applier) fn log_end(
        &mut self,
        window: WindowId,
        ending: Ending,
        placed: Option<Placed>,
        now: Instant,
    ) {
        self.log_step();
        let lines = self.transitions.journal.ended(window, ending, placed, now);
        if placed == Some(Placed::Landing) && !lines.is_empty() {
            self.log_landing(window, now);
        }
        for line in lines {
            self.log(line);
        }
    }

    /// Has the log hear when `window` lands where the placement posted to
    /// it at `now`, which ended a transition, takes it.
    pub(in crate::tiler::applier) fn log_landing(&mut self, window: WindowId, now: Instant) {
        if let Some(landing) = self.posted.placed.get_mut(&window) {
            landing.logged = Some(now);
        }
    }

    /// Logs that `window` landed `how` at `now`, if the log hears of it: the
    /// placement it landed for, `posted` then, ended a transition.
    pub(in crate::tiler::applier) fn log_landed(
        &mut self,
        window: WindowId,
        posted: Option<Instant>,
        how: Landed,
        now: Instant,
    ) {
        if let Some(posted) = posted {
            let after = now.saturating_duration_since(posted);
            self.log(LandedLine { window, after, how });
        }
    }

    /// Logs `line` for debugging.
    pub(in crate::tiler::applier) fn log(&mut self, line: impl fmt::Display) {
        crate::log_debug!("{line}");
        #[cfg(test)]
        self.transitions.logged.push(line.to_string());
    }

    /// The lines logged about transitions since the last time this was
    /// asked, for the tests to read.
    #[cfg(test)]
    pub(in crate::tiler) fn take_transition_lines(&mut self) -> Vec<String> {
        std::mem::take(&mut self.transitions.logged)
    }
}
