//! The frame step: the frames of the moving windows, what the platform
//! found of each, and the motions that are over.
//!
//! Every frame due is posted in one call. A window that handled the frame
//! before moves on, and its border follows it; one that has not stays, and
//! so does its border. A motion is over at its end time, or once a window
//! of its step has fallen behind: the window is then placed at its target
//! in full, which the platform orders behind the frames posted to it
//! before, and its border is drawn there. A window maximized, minimized or
//! gone on its way stops where it is, and what was kept for its border, and
//! for its last placement to land, is dropped with its motion. The frame
//! step also ends the waits by time for windows to land where a posted
//! placement takes them (`applier_landing.rs`).
//!
//! The log hears of a window that falls behind, of each motion that ends
//! and of each step whose last motion ended, never of a frame on its own
//! (see `motion::Journal`).

use std::time::Instant;

use crate::motion::Ending;

use super::super::Platform;
use super::Applier;

impl Applier {
    /// When a frame is due even if no display frame comes, after `after`,
    /// the last frame step: when the first motion is due to end, or the
    /// first wait by time for a window to land ends. `None` while nothing
    /// is due.
    pub(in crate::tiler) fn frame_due(&self, after: Option<Instant>) -> Option<Instant> {
        let motions = self.transitions.motions.deadline();
        let due = motions.into_iter().chain(self.landing_due()).min();
        due.filter(|&due| after.is_none_or(|after| due > after))
    }

    /// The frame step at `now`: posts the frames due, acts on what the
    /// platform found, places the windows whose motions are over, and ends
    /// the waits by time that are due.
    pub(in crate::tiler) fn frame(&mut self, platform: &mut dyn Platform, now: Instant) {
        self.log_step();
        let mut tick = self.transitions.motions.frame(now);
        // The motions whose end time has come. Those `take` adds after them
        // end as their step fell behind.
        let on_time = tick.finished.len();
        if !tick.in_flight.is_empty() {
            let transitions = &mut self.transitions;
            let outcomes = &mut transitions.outcomes;
            match platform.control().place_frames(&tick.in_flight, outcomes) {
                Ok(()) => {
                    transitions.frames_failing = false;
                    let ended = transitions.journal.framed(&tick.in_flight, outcomes, now);
                    transitions.motions.take(&mut tick, outcomes, now);
                    self.posted.frames_sent(&tick.in_flight, outcomes);
                    ended.into_iter().for_each(|line| self.log(line));
                    for &window in &tick.behind {
                        if let Some(line) = self.transitions.journal.fell_behind(window) {
                            self.log(line);
                        }
                    }
                }
                Err(error) => self.frames_failed(&error, &tick, platform, now),
            }
        }
        self.follow_reached(&tick.reached, platform.decorations());
        for &window in &tick.stopped {
            self.forget_kept_border(window);
            self.forget_landing(window);
        }
        for (i, &(window, target)) in tick.finished.iter().enumerate() {
            let ending = if i < on_time {
                Ending::AtTarget
            } else {
                Ending::FellBehind
            };
            let placed = self.in_place(window, target, platform);
            self.log_end(window, ending, Some(placed), now);
        }
        self.land_overdue(platform, now);
    }
}
