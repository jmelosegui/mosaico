use crate::action::Origin;
use crate::config::{Config, WindowRule};
use crate::{Action, WindowEvent};

use super::Display;

/// Something the tiler reacts to.
#[derive(Debug, Clone)]
pub enum Input {
    /// A user action, from a hotkey, the CLI or the pointer on a bar, and
    /// where it comes from, which decides the monitor it acts on and
    /// whether the pointer may move for it.
    Action(Action, Origin),
    /// A window event, already checked against
    /// [`Expectations`](super::Expectations) so it is not an echo of our
    /// own effects. Display and work area events are ignored here; send
    /// [`Input::Displays`] or [`Input::WorkAreas`] instead.
    Event(WindowEvent),
    /// A display change was noted. The monitors are read once it settles,
    /// and sent as [`Input::Displays`].
    DisplayChanging,
    /// The set of monitors changed.
    Displays {
        /// Every monitor now connected, left to right as
        /// [`Enumerator::monitors`](super::Enumerator::monitors) lists them.
        monitors: Vec<Display>,
        /// Space to keep free for the bar.
        bar: BarReservation,
    },
    /// The monitors are the same, but their work areas may have moved,
    /// for example because the taskbar was shown or hidden.
    WorkAreas {
        /// Fresh work areas for every monitor, in the order
        /// [`Enumerator::monitors`](super::Enumerator::monitors) lists
        /// them: a monitor that matches by neither key nor session id is
        /// matched by its position.
        monitors: Vec<Display>,
        /// Space to keep free for the bar.
        bar: BarReservation,
    },
    /// The bar changed size or moved to other monitors.
    Bar(BarReservation),
    /// The configuration file changed.
    Config(Box<Config>),
    /// The window rules changed.
    Rules(Vec<WindowRule>),
    /// Redraw every border from the current state.
    RefreshBorders,
    /// Repaint every visible window, including the ones already in
    /// place. Repairs applications left drawing at a stale offset.
    RepaintAll,
    /// The tiler is stopping. Shows every window it hid and gives every
    /// window its normal frame back.
    Shutdown,
}

/// Space reserved at the top of monitors for the bar.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BarReservation {
    /// Bar height in pixels, or 0 for no bar.
    pub height: i32,
    /// Indices of the monitors that show a bar. Empty means all.
    pub monitors: Vec<usize>,
}
