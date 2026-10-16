//! What ends the daemon loop's waits, and what the loop is told to do.

use crate::WindowEvent;

/// A message the daemon loop received.
#[derive(Debug, Clone, PartialEq)]
pub enum Message<T> {
    /// A window event. Window events are batched, see [`Step::Events`].
    Event(WindowEvent),
    /// A display frame, from the [`FrameClock`](super::FrameClock) while
    /// the loop wants frames. Frames are folded into one
    /// [`Step::Frame`].
    Frame,
    /// Anything else, such as a user action, a CLI command, a config
    /// reload or a timer tick. The scheduler only decides when it runs.
    Task(T),
}

impl<T> Message<T> {
    /// Whether the message is input, which keeps the effects the tiling
    /// manager holds back waiting. A frame is not, nor is a window's
    /// report that its position changed: those come many times a second
    /// while a window moves, whoever moves it, the tiler included, and
    /// say nothing about whether the user is done.
    fn is_input(&self) -> bool {
        !matches!(
            self,
            Self::Frame | Self::Event(WindowEvent::LocationChanged { .. })
        )
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

    /// The messages that ended the wait, sorted by what runs them.
    pub(super) fn into_batch(self) -> Batch<T> {
        let mut batch = Batch {
            tasks: Vec::new(),
            events: Vec::new(),
            frame: false,
        };
        if let Self::Received(messages) = self {
            for message in messages {
                match message {
                    Message::Event(event) => batch.events.push(event),
                    Message::Frame => batch.frame = true,
                    Message::Task(task) => batch.tasks.push(task),
                }
            }
        }
        batch
    }
}

/// The messages of a wake, sorted by what runs them.
pub(super) struct Batch<T> {
    /// The tasks, in arrival order.
    pub(super) tasks: Vec<T>,
    /// The window events, in arrival order.
    pub(super) events: Vec<WindowEvent>,
    /// Whether any display frame came.
    pub(super) frame: bool,
}

impl<T> Batch<T> {
    /// Takes the display changes out of the events, and says whether
    /// there were any.
    pub(super) fn take_display_changes(&mut self) -> bool {
        let before = self.events.len();
        self.events
            .retain(|event| !matches!(event, WindowEvent::DisplayChanged));
        self.events.len() != before
    }
}

/// Something the daemon loop has to do, in the order
/// [`Scheduler::step`](super::Scheduler::step) returns them.
#[derive(Debug, Clone, PartialEq)]
pub enum Step<T> {
    /// Apply the effects the tiler is holding back.
    Flush,
    /// The display configuration changed, and is rebuilt for once it has
    /// settled ([`RebuildMonitors`](Self::RebuildMonitors)): stop what moves
    /// to where the old layout sent it.
    DisplayChanging,
    /// The display configuration has settled since it last changed:
    /// read the monitors again and rebuild for them.
    RebuildMonitors,
    /// Run a task.
    Run(T),
    /// Handle the window events of a batch, in order.
    Events(Vec<WindowEvent>),
    /// Move what is in motion on to where it is at the time this step
    /// runs: frames arrived, or a motion is due to end.
    Frame,
}
