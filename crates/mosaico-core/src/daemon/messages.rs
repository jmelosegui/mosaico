//! What ends the daemon loop's waits, and what the loop is told to do.

use crate::WindowEvent;

/// A message the daemon loop received.
#[derive(Debug, Clone, PartialEq)]
pub enum Message<T> {
    /// A window event. Window events are batched, see [`Step::Events`].
    Event(WindowEvent),
    /// Anything else, such as a user action, a CLI command, a config
    /// reload or a timer tick. The scheduler only decides when it runs.
    Task(T),
}

impl<T> Message<T> {
    /// Whether the message is input, which keeps the effects the tiling
    /// manager holds back waiting. A window's report that its position
    /// changed is not: those come many times a second while a window
    /// moves, whoever moves it, the tiler included, and say nothing about
    /// whether the user is done.
    fn is_input(&self) -> bool {
        !matches!(self, Self::Event(WindowEvent::LocationChanged { .. }))
    }
}

/// What ended the daemon loop's wait for messages.
#[derive(Debug, Clone, PartialEq)]
pub enum Wake<T> {
    /// Messages arrived: everything that was queued, oldest first.
    Received(Vec<Message<T>>),
    /// The wait ran out with nothing received.
    TimedOut,
}

impl<T> Wake<T> {
    /// Whether any of the messages that ended the wait is input.
    pub(super) fn brought_input(&self) -> bool {
        match self {
            Self::Received(messages) => messages.iter().any(Message::is_input),
            Self::TimedOut => false,
        }
    }
}

/// Something the daemon loop has to do, in the order
/// [`Scheduler::step`](super::Scheduler::step) returns them.
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
