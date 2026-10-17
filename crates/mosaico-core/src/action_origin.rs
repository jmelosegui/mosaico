//! Where an action comes from.

/// Where an action comes from, which decides the monitor it acts on and
/// whether the pointer may move for it.
///
/// Actions that act on a monitor (cycling the layout, monocle and the
/// workspace switches) act on the monitor the origin names. The others,
/// such as focusing or closing a window, act on the focused window
/// whatever their origin.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Origin {
    /// A hotkey or the command line. The action acts on the focused
    /// monitor, and the pointer follows focus as `[mouse] follows_focus`
    /// says.
    #[default]
    Focused,
    /// A click or a scroll on the bar of the monitor at this index, in the
    /// tiler's order of the monitors. An action that acts on a monitor
    /// acts on this one, which takes focus first when it does not have
    /// it. Nothing the action does moves the pointer.
    Bar(usize),
}

impl Origin {
    /// The monitor the origin names, if it names one.
    pub fn monitor(self) -> Option<usize> {
        match self {
            Self::Focused => None,
            Self::Bar(monitor) => Some(monitor),
        }
    }

    /// Whether the action comes from the pointer, which then stays where
    /// it is.
    pub fn from_pointer(self) -> bool {
        matches!(self, Self::Bar(_))
    }
}
