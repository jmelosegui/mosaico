use crate::action::Origin;
use crate::config::{Config, WindowRule};
use crate::{Action, WindowEvent, WindowId};

use super::{Display, Reach};

/// Something the tiler reacts to.
#[derive(Debug, Clone)]
pub enum Input {
    /// A user action, from a hotkey, the CLI or the pointer on a bar, and
    /// where it comes from, which decides the monitor it acts on and
    /// whether the pointer may move for it.
    Action(Action, Origin),
    /// Focus a window the tiler manages and bring it into view, switching
    /// its monitor to its workspace and stepping out of monocle when
    /// either keeps it out of sight. For a launch binding that focuses an
    /// open window of its application.
    Raise(WindowId),
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
        /// How far the bar reaches into each of them, by the ids they
        /// get, as [`Tiler::places_for`](super::Tiler::places_for) has
        /// them.
        reach: Reach,
    },
    /// The monitors are the same, but their work areas may have moved,
    /// for example because the taskbar was shown or hidden.
    WorkAreas {
        /// Fresh work areas for every monitor, in the order
        /// [`Enumerator::monitors`](super::Enumerator::monitors) lists
        /// them: a monitor that matches by neither key nor session id is
        /// matched by its position.
        monitors: Vec<Display>,
        /// How far the bar reaches into each monitor.
        reach: Reach,
    },
    /// The bar changed how far it reaches into the monitors.
    Reach(Reach),
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
