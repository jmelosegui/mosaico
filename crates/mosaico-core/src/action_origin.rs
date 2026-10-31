//! Where an action comes from, and what it acts on.

use super::Action;

/// Where an action comes from, which decides the monitor it acts on and
/// whether the pointer may move for it.
///
/// Actions that act on a monitor ([`Scope::Monitor`]) act on the monitor
/// the origin names. The others act on the focused window, or on the
/// whole daemon, whatever their origin.
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

/// What an action acts on, which decides where it may come from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// The focused window: focusing or moving from it, sending it to
    /// another workspace, closing or minimizing it. A click on the bar of
    /// one monitor would act on a window of another, so a bar never runs
    /// these.
    FocusedWindow,
    /// A monitor: its layout, monocle and the workspace on screen. From a
    /// bar, the bar's monitor.
    Monitor,
    /// The whole daemon: laying out every monitor again, and pausing the
    /// hotkeys.
    Daemon,
}

impl Action {
    /// What the action acts on.
    pub fn scope(self) -> Scope {
        match self {
            Self::Focus(_)
            | Self::Move(_)
            | Self::SendToWorkspace(_)
            | Self::CloseFocused
            | Self::MinimizeFocused
            | Self::ToggleFloat
            | Self::GrowWidth
            | Self::ShrinkWidth
            | Self::GrowHeight
            | Self::ShrinkHeight => Scope::FocusedWindow,
            Self::CycleLayout
            | Self::ToggleMonocle
            | Self::CycleFloat
            | Self::GoToWorkspace(_)
            | Self::GoToWorkspaceNext
            | Self::GoToWorkspacePrev => Scope::Monitor,
            Self::Retile | Self::TogglePause => Scope::Daemon,
        }
    }
}

#[cfg(test)]
#[path = "action_scope_tests.rs"]
mod scope_tests;
