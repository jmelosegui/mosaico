//! Decides what the daemon loop runs, and when.

use std::time::{Duration, Instant};

use crate::tiler::release_at;

use super::dedup::dedup;
use super::messages::{Step, Wake};
use super::settle::DisplaySettle;

#[path = "scheduler_frame.rs"]
mod frame;

/// Decides what the daemon loop runs, and when.
///
/// Before each wait for messages the loop calls [`Scheduler::begin_wait`]
/// with the time, what
/// [`Coalescer::held_since`](crate::tiler::Coalescer::held_since)
/// reports and the first frame step due after the last one, as at the end
/// of a motion, then hands whatever ended the wait to [`Scheduler::step`].
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
    /// A display change waiting for the configuration to settle.
    settle: DisplaySettle,
}

impl Scheduler {
    /// Starts a wait at `now`, once the loop has run every step it was
    /// given, unless the last wait goes on, and returns how long it may
    /// wait for messages, or `None` when nothing is due until one
    /// arrives. `held_since` is when the effects the tiling manager holds
    /// started being held, if any, and `frame_due` the first frame step due
    /// after the [`last_frame_step`](Self::last_frame_step), as at the end of
    /// a motion: the wait ends by then with it. One at or before it is let go.
    pub fn begin_wait(
        &mut self,
        held_since: Option<Instant>,
        frame_due: Option<Instant>,
        now: Instant,
    ) -> Option<Duration> {
        let waiting_since = *self.waiting_since.get_or_insert(now);
        self.hold = held_since.map(|since| match self.hold {
            Some((seen, first_wait)) if seen == since => (since, first_wait),
            _ => (since, now),
        });
        self.release = self
            .hold
            .map(|(_, first_wait)| release_at(first_wait, waiting_since));
        self.frame_due = self.not_stepped(frame_due);
        let due = if self.settle.is_due(now) {
            Some(Duration::ZERO)
        } else {
            match self.release {
                Some(release) => Some(release.saturating_duration_since(now)),
                None => self.settle.remaining(now),
            }
        };
        let frame = self.frame_due.map(|due| due.saturating_duration_since(now));
        due.into_iter().chain(frame).min()
    }

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
        let mut batch = wake.into_batch();
        dedup(&mut batch.events);
        let changing = batch.take_display_changes();
        let flush = release.is_some_and(|release| due_by >= release);
        let rebuild = self.settle.take_due(due_by);
        if changing {
            self.settle.note(now);
        }
        if input || flush || rebuild {
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
