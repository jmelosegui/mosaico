//! Decides what the daemon loop runs, and when.

use std::time::{Duration, Instant};

use crate::WindowEvent;
use crate::tiler::release_at;

use super::dedup::dedup;
use super::settle::DisplaySettle;

/// A message the daemon loop received.
#[derive(Debug, Clone, PartialEq)]
pub enum Message<T> {
    /// A window event. Window events are batched, see [`Step::Events`].
    Event(WindowEvent),
    /// Anything else, such as a user action, a CLI command, a config
    /// reload or a timer tick. The scheduler only decides when it runs.
    Task(T),
}

/// What ended the daemon loop's wait for messages.
#[derive(Debug, Clone, PartialEq)]
pub enum Wake<T> {
    /// Messages arrived: everything that was queued, oldest first.
    Received(Vec<Message<T>>),
    /// The wait ran out with nothing received.
    TimedOut,
}

/// Something the daemon loop has to do, in the order
/// [`Scheduler::step`] returns them.
#[derive(Debug, Clone, PartialEq)]
pub enum Step<T> {
    /// Apply the effects the tiler is holding back.
    Flush,
    /// The display configuration has settled since it last changed:
    /// read the monitors again and rebuild for them.
    RebuildMonitors,
    /// Run a task.
    Run(T),
    /// Handle the window events of a batch, in order.
    Events(Vec<WindowEvent>),
}

/// Decides what the daemon loop runs, and when.
///
/// Before each wait for messages the loop calls [`Scheduler::begin_wait`]
/// with the time and what
/// [`Coalescer::held_since`](crate::tiler::Coalescer::held_since)
/// reports, then hands whatever ended the wait to [`Scheduler::step`].
///
/// - Held effects are flushed once the loop has waited
///   [`QUIET`](crate::tiler::QUIET) with nothing arriving, and at the
///   latest [`MAX_HOLD`](crate::tiler::MAX_HOLD) after it first waited
///   with them held. Waits only start once everything before them has
///   run, so a slow batch, a rebuild or startup never counts as quiet or
///   toward the cap. A message ends the wait without a flush, unless the
///   cap had already passed.
/// - A display change is rebuilt for once it has been quiet for
///   [`SETTLE`](super::SETTLE). While effects are held the wait is for
///   them alone, so a change that settles meanwhile follows the flush.
/// - What was due when the wait began runs before the messages that
///   ended it: first the flush, then the rebuild.
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
    /// A display change waiting for the configuration to settle.
    settle: DisplaySettle,
}

impl Scheduler {
    /// Starts a wait at `now`, once the loop has run every step it was
    /// given, and returns how long it may wait for messages, or `None`
    /// when nothing is due until one arrives. `held_since` is when the
    /// effects the tiling manager holds started being held, if it holds
    /// any.
    pub fn begin_wait(&mut self, held_since: Option<Instant>, now: Instant) -> Option<Duration> {
        self.waiting_since = Some(now);
        self.hold = held_since.map(|since| match self.hold {
            Some((seen, first_wait)) if seen == since => (since, first_wait),
            _ => (since, now),
        });
        self.release = self.hold.map(|(_, first_wait)| release_at(first_wait, now));
        if self.settle.is_due(now) {
            return Some(Duration::ZERO);
        }
        match self.release {
            Some(release) => Some(release.saturating_duration_since(now)),
            None => self.settle.remaining(now),
        }
    }

    /// Takes what ended the wait at `now`, and returns what to do, in
    /// order.
    pub fn step<T>(&mut self, wake: Wake<T>, now: Instant) -> Vec<Step<T>> {
        let waiting_since = self.waiting_since.take();
        let release = self.release.take();
        // Messages end the wait before anything else comes due, so they
        // only find what was due already when it began.
        let due_by = match wake {
            Wake::TimedOut => now,
            Wake::Received(_) => waiting_since.unwrap_or(now),
        };

        let mut steps = Vec::new();
        if release.is_some_and(|release| due_by >= release) {
            steps.push(Step::Flush);
        }
        if self.settle.take_due(due_by) {
            steps.push(Step::RebuildMonitors);
        }
        if let Wake::Received(messages) = wake {
            let mut events = Vec::new();
            for message in messages {
                match message {
                    Message::Event(event) => events.push(event),
                    Message::Task(task) => steps.push(Step::Run(task)),
                }
            }
            dedup(&mut events);
            if take_display_changes(&mut events) {
                self.settle.note(now);
            }
            if !events.is_empty() {
                steps.push(Step::Events(events));
            }
        }
        steps
    }
}

/// Takes the display changes out of `events`, and says whether there
/// were any.
fn take_display_changes(events: &mut Vec<WindowEvent>) -> bool {
    let before = events.len();
    events.retain(|event| !matches!(event, WindowEvent::DisplayChanged));
    events.len() != before
}

#[cfg(test)]
#[path = "tests/scheduler_tests.rs"]
mod tests;
