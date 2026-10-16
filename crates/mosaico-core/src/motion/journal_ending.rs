//! How a motion ends, as the log tells it.

use std::fmt;

/// How a motion ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ending {
    /// It ran to its end time, and its window was placed at its target.
    AtTarget,
    /// Its window stopped where it was.
    Stopped(Stop),
    /// The displays changed, and its window stopped where it was. It keeps
    /// its target, where it is placed once the change is rebuilt for,
    /// unless something places it or takes it out of view meanwhile.
    DisplaysChanged,
    /// Its window is gone.
    Destroyed,
    /// A window of its step fell behind, and it went straight to its
    /// target.
    FellBehind,
    /// A placement that snaps ended it, wherever it put the window: one
    /// that does not animate, as one to another monitor.
    NewTarget,
    /// The tiler let go of its window, and it went straight to its target.
    LetGo,
    /// Its window was revealed, and it went straight to its target.
    Revealed,
    /// The daemon stopping sent it straight to its target.
    Shutdown,
    /// Posting its frames failed, and it went straight to its target.
    FramesFailed,
    /// Transitions turned off, for a platform without frames or a daemon
    /// without display frames, and it went straight to its target.
    TransitionsOff,
}

/// Why a window stopped where it was, on its way.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stop {
    /// It was hidden.
    Hidden,
    /// It was minimized.
    Minimized,
    /// It was maximized.
    Maximized,
    /// The user started dragging it.
    Grabbed,
}

impl fmt::Display for Ending {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let straight = "went straight to its target";
        match self {
            Self::AtTarget => f.write_str("ended at its target"),
            Self::Stopped(stop) => write!(f, "stopped where it was, {stop}"),
            Self::DisplaysChanged => {
                f.write_str("stopped where it was, keeping its target, as the displays changed")
            }
            Self::Destroyed => f.write_str("ended, its window gone"),
            Self::FellBehind => write!(f, "{straight}, its step fell behind"),
            Self::NewTarget => f.write_str("ended with a placement that snaps"),
            Self::LetGo => write!(f, "{straight}, let go of by the tiler"),
            Self::Revealed => write!(f, "{straight}, revealed"),
            Self::Shutdown => write!(f, "{straight} as the daemon stopped"),
            Self::FramesFailed => write!(f, "{straight}, its frames failed"),
            Self::TransitionsOff => write!(f, "{straight}, transitions turned off"),
        }
    }
}

impl fmt::Display for Stop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Hidden => "hidden",
            Self::Minimized => "minimized",
            Self::Maximized => "maximized",
            Self::Grabbed => "grabbed by the user",
        })
    }
}

/// How the placement that ends a transition was made.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placed {
    /// Made at once.
    AtOnce,
    /// Posted behind what was on its way to the window, which was there
    /// already, with nothing sent to it before that takes it elsewhere
    /// first: nothing is waited for.
    PostedThere,
    /// Posted behind what was on its way to the window, which is still to
    /// land there: a [`LandedLine`](super::LandedLine) says how and when.
    Landing,
}

impl Placed {
    /// What a line adds of a placement made this way.
    pub(super) fn note(self) -> &'static str {
        match self {
            Self::AtOnce => "",
            Self::PostedThere => {
                "; its placement posted behind what was on its way, the window there already"
            }
            Self::Landing => "; its placement posted behind what was on its way",
        }
    }
}

#[cfg(test)]
#[path = "tests/journal_ending_tests.rs"]
mod tests;
