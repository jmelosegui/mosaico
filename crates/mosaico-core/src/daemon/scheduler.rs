//! Decides what the daemon loop runs, and when.

use std::time::Instant;

use super::dedup::dedup;
use super::messages::{Step, Wake};
use super::settle::DisplaySettle;

#[path = "scheduler_frame.rs"]
mod frame;
#[path = "scheduler_wait.rs"]
mod wait;

/// Decides what the daemon loop runs, and when.
///
/// Before each wait for messages the loop calls [`Scheduler::begin_wait`]
/// with the time, what
/// [`Coalescer::held_since`](crate::tiler::Coalescer::held_since)
/// reports, the first frame step due after the last one, as at the end of
/// a motion, and when the wheel's turns on a bar are due, then hands
/// whatever ended the wait to [`Scheduler::step`].
///
/// - Held effects are flushed once the loop has waited
///   [`QUIET`](crate::tiler::QUIET) with no input arriving, and at the
///   latest [`MAX_HOLD`](crate::tiler::MAX_HOLD) after it first waited
///   with them held. Waits only start once everything before them has
///   run, so a slow batch of input, a rebuild or startup never counts as
///   quiet or toward the cap. Input ends the wait without a flush, unless
///   the cap had already passed.
/// - Neither a display frame nor a window's report that its position
///   changed is input, whoever moved the window: a window that moves
///   reports it many times a second, the tiler's own placements included.
///   A wake that brought only those counts like a timeout: it finds what
///   is due by then, and unless something was, the wait goes on, through
///   the batch that handles them, so the time that batch takes counts as
///   quiet. So frames and windows moving on screen never push held
///   effects back to the cap, and they are flushed once the quiet window
///   has passed.
/// - The frame step comes once per wake that brought frames, however
///   many, and once the wait reaches the end of a motion, so motions end
///   on time even when no frame comes. It runs last, after the tasks and
///   events of the wake. The loop passes the earliest end of a motion
///   after the last frame step, and an end at or before it is let go, so
///   one left in the past never makes the loop spin.
/// - A display change is rebuilt for once it has been quiet for
///   [`SETTLE`](super::SETTLE). While effects are held the wait is for
///   them alone, so a change that settles meanwhile follows the flush.
/// - A display change in the wake comes first, to stop what moves. Then
///   what was due when the wait began runs, before the messages that ended
///   it: the flush, then the rebuild, after which the change is noted again
///   as it still settles. A wait that ran either is over, as one input ends.
/// - Tasks run next, in arrival order, so hotkeys and commands stay
///   responsive while windows flood the queue. The batch's window events
///   come last, as one step, keeping only the last position change and
///   the last title change of each window, and without display changes.
/// - The wheel step comes once the wheel's turns are due, whatever ended
///   the wait, after the tasks and events of the wake, so the turns and
///   clicks they bring count first, and before the frame step. It runs
///   after a flush of the same wake, and ends the wait as input does: it
///   switches workspaces, which takes time that must not count as quiet.
#[derive(Debug, Default)]
pub struct Scheduler {
    /// When the current wait began.
    waiting_since: Option<Instant>,
    /// When what is held during the current wait is due.
    release: Option<Instant>,
    /// The hold the loop last waited with: when it began, and when the
    /// loop first waited with it.
    hold: Option<(Instant, Instant)>,
    /// When the current wait has to end for the frame step.
    frame_due: Option<Instant>,
    /// When the last frame step was taken: the end of a motion by then
    /// has had its step.
    frame_stepped: Option<Instant>,
    /// When the current wait has to end for the wheel step.
    wheel_due: Option<Instant>,
    /// A display change waiting for the configuration to settle.
    settle: DisplaySettle,
}

impl Scheduler {
    /// Takes what ended the wait at `now`, and returns what to do, in
    /// order.
    pub fn step<T>(&mut self, wake: Wake<T>, now: Instant) -> Vec<Step<T>> {
        let release = self.release.take();
        // Input ends the wait before anything else comes due, so it only finds
        // what was due when the wait began; anything else, what is due by now.
        let input = wake.brought_input();
        let due_by = match self.waiting_since {
            Some(since) if input => since,
            _ => now,
        };
        let motion_due = self.frame_due.take().is_some_and(|due| now >= due);
        let wheel = self.wheel_due.take().is_some_and(|due| now >= due);
        let mut batch = wake.into_batch();
        dedup(&mut batch.events);
        let changing = batch.take_display_changes();
        let flush = release.is_some_and(|release| due_by >= release);
        let rebuild = self.settle.take_due(due_by);
        if changing {
            self.settle.note(now);
        }
        if input || flush || rebuild || wheel {
            self.waiting_since = None;
        }
        let mut steps = Vec::new();
        steps.extend(changing.then_some(Step::DisplayChanging));
        steps.extend(flush.then_some(Step::Flush));
        steps.extend(rebuild.then_some(Step::RebuildMonitors));
        steps.extend((changing && rebuild).then_some(Step::DisplayChanging));
        steps.extend(batch.tasks.into_iter().map(Step::Run));
        if !batch.events.is_empty() {
            steps.push(Step::Events(batch.events));
        }
        steps.extend(wheel.then_some(Step::Wheel));
        if batch.frame || motion_due {
            self.frame_stepped = Some(now);
            steps.push(Step::Frame);
        }
        steps
    }
}

#[cfg(test)]
#[path = "tests/scheduler_tests.rs"]
mod tests;
