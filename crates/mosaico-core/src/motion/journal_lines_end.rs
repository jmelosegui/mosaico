//! The lines the log gets as windows fall behind, and as motions and
//! steps end.

use std::fmt;
use std::time::Duration;

use crate::WindowId;

use super::{Ending, Placed};

/// The line of a window that fell behind, which ends every motion of its
/// step at once.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BehindLine {
    /// The window.
    pub window: WindowId,
    /// Which frame of its motion found it so, counting from 1.
    pub frame: u32,
}

impl fmt::Display for BehindLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "transition: {} fell behind at its frame {}: its step goes straight to its targets",
            self.window, self.frame
        )
    }
}

/// The line of a motion that ended, with what its frames found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndLine {
    /// The window that moved.
    pub window: WindowId,
    /// How the motion ended.
    pub ending: Ending,
    /// How long it ran.
    pub took: Duration,
    /// How long it was to run, to the end of its latest target.
    pub planned: Duration,
    /// The frames sent for its window: every one given to `place_frames`,
    /// posted to it or not.
    pub sent: u32,
    /// How many of them found the window had handled the frame before:
    /// posted to it, or not needed or refused by the system.
    pub handled: u32,
    /// How many found it still to handle the frame before.
    pub pending: u32,
    /// How the placement that ended it was made, if one did.
    pub placed: Option<Placed>,
}

impl fmt::Display for EndLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (took, planned) = (self.took.as_millis(), self.planned.as_millis());
        write!(
            f,
            "transition: {} {} after {took} ms of {planned}: ",
            self.window, self.ending
        )?;
        frames(f, self.sent, self.handled, self.pending)?;
        f.write_str(self.placed.map_or("", Placed::note))
    }
}

/// The line of a step whose last motion has ended, with what the frames
/// of all of them found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepDoneLine {
    /// The windows of its motions, in the order they ended.
    pub windows: Vec<WindowId>,
    /// The frames sent for them.
    pub sent: u32,
    /// How many of them found their window had handled the frame before.
    pub handled: u32,
    /// How many found it still to handle the frame before.
    pub pending: u32,
    /// The window found pending the most, and how often, if any was.
    pub slowest: Option<(WindowId, u32)>,
}

impl fmt::Display for StepDoneLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let count = self.windows.len();
        write!(f, "transition: a step of {count} window(s) is done:")?;
        for (i, window) in self.windows.iter().enumerate() {
            let comma = if i == 0 { "" } else { "," };
            write!(f, "{comma} {window}")?;
        }
        f.write_str("; ")?;
        frames(f, self.sent, self.handled, self.pending)?;
        match self.slowest {
            Some((window, pending)) => write!(f, "; slowest {window}, {pending} pending"),
            None => Ok(()),
        }
    }
}

/// Writes what the frames of one motion or one step found.
fn frames(f: &mut fmt::Formatter<'_>, sent: u32, handled: u32, pending: u32) -> fmt::Result {
    write!(f, "{sent} frame(s) sent, {handled} handled, ")?;
    match pending {
        0 => f.write_str("none found pending"),
        n => write!(f, "{n} found pending"),
    }
}

/// A line the log gets as a motion ends.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EndingLine {
    /// The motion's own.
    Motion(EndLine),
    /// Its step's, as it was the last of the step to end.
    Step(StepDoneLine),
}

impl fmt::Display for EndingLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Motion(line) => line.fmt(f),
            Self::Step(line) => line.fmt(f),
        }
    }
}

#[cfg(test)]
#[path = "tests/journal_lines_end_tests.rs"]
mod tests;
