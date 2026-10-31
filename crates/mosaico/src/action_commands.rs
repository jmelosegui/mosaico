//! The `action` subcommands: the actions the daemon carries out, as the
//! command line names them.

use clap::Subcommand;

use mosaico_core::Action;
use mosaico_core::action::Direction;

#[derive(Subcommand)]
pub(crate) enum DirectionCommands {
    /// Left
    Left,
    /// Right
    Right,
    /// Up
    Up,
    /// Down
    Down,
}

#[derive(Subcommand)]
pub(crate) enum ActionCommands {
    /// Focus a window in the given direction
    Focus {
        #[command(subcommand)]
        direction: DirectionCommands,
    },
    /// Move the focused window in the given direction
    Move {
        #[command(subcommand)]
        direction: DirectionCommands,
    },
    /// Re-apply the current layout
    Retile,
    /// Toggle monocle mode (focused window fills the monitor)
    ToggleMonocle,
    /// Close the currently focused window
    CloseFocused,
    /// Cycle to the next layout on the focused workspace
    CycleLayout,
    /// Switch to workspace N (1-8) on the focused monitor
    GoToWorkspace {
        /// Workspace number (1-8)
        n: u8,
    },
    /// Switch to the workspace after the one on screen on the focused
    /// monitor
    #[command(alias = "goto-workspace-next")]
    GoToWorkspaceNext,
    /// Switch to the workspace before the one on screen on the focused
    /// monitor
    #[command(alias = "goto-workspace-prev")]
    GoToWorkspacePrev,
    /// Send the focused window to workspace N (1-8)
    SendToWorkspace {
        /// Workspace number (1-8)
        n: u8,
    },
    /// Float the focused window above the tiles, or tile it again
    ToggleFloat,
    /// Focus the next floating window on the focused monitor
    CycleFloat,
    /// Widen the focused floating window
    GrowWidth,
    /// Narrow the focused floating window
    ShrinkWidth,
    /// Heighten the focused floating window
    GrowHeight,
    /// Shorten the focused floating window
    ShrinkHeight,
}

impl ActionCommands {
    /// The action the subcommand names.
    pub(crate) fn action(self) -> Action {
        match self {
            Self::Focus { direction: d } => Action::Focus(direction(d)),
            Self::Move { direction: d } => Action::Move(direction(d)),
            Self::Retile => Action::Retile,
            Self::ToggleMonocle => Action::ToggleMonocle,
            Self::CloseFocused => Action::CloseFocused,
            Self::CycleLayout => Action::CycleLayout,
            Self::GoToWorkspace { n } => Action::GoToWorkspace(n),
            Self::GoToWorkspaceNext => Action::GoToWorkspaceNext,
            Self::GoToWorkspacePrev => Action::GoToWorkspacePrev,
            Self::SendToWorkspace { n } => Action::SendToWorkspace(n),
            Self::ToggleFloat => Action::ToggleFloat,
            Self::CycleFloat => Action::CycleFloat,
            Self::GrowWidth => Action::GrowWidth,
            Self::ShrinkWidth => Action::ShrinkWidth,
            Self::GrowHeight => Action::GrowHeight,
            Self::ShrinkHeight => Action::ShrinkHeight,
        }
    }
}

fn direction(d: DirectionCommands) -> Direction {
    match d {
        DirectionCommands::Left => Direction::Left,
        DirectionCommands::Right => Direction::Right,
        DirectionCommands::Up => Direction::Up,
        DirectionCommands::Down => Direction::Down,
    }
}

#[cfg(test)]
#[path = "action_commands_tests.rs"]
mod tests;
