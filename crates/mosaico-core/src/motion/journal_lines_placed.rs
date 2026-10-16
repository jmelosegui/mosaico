//! The lines the log gets as a window is placed once its transition is
//! over: at the target a display change kept for it, and where a placement
//! posted behind what was on its way took it, once it lands there.

use std::fmt;
use std::time::Duration;

use crate::WindowId;

use super::Placed;

/// When a window a display change stopped is placed at the target it
/// kept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kept {
    /// Once the change is rebuilt for, or there is none.
    ChangeOver,
    /// As the daemon stops first.
    Shutdown,
}

/// The line of a window a display change stopped, placed at the target it
/// kept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeptLine {
    /// The window.
    pub window: WindowId,
    /// When it was placed.
    pub when: Kept,
    /// How its placement was made.
    pub placed: Placed,
}

impl fmt::Display for KeptLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let when = match self.when {
            Kept::ChangeOver => ", the display change over",
            Kept::Shutdown => " as the daemon stopped",
        };
        write!(
            f,
            "transition: {} placed at the target it kept{when}{}",
            self.window,
            self.placed.note()
        )
    }
}

/// How a window a transition placed behind what was on its way to it came
/// to be taken as landed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Landed {
    /// It reported being where it was placed, no edge of it more than
    /// [`MIN_TRAVEL`](super::super::MIN_TRAVEL) pixels away.
    There,
    /// It reported being elsewhere while nothing posted to it may still be
    /// on its way.
    Away,
    /// It reported being elsewhere once it was no longer waited for by
    /// time, as it had not moved by then, and is taken to be where it was
    /// placed.
    Late,
    /// It had not reported being there by the end of the wait for it by
    /// time, but had moved since the placement was posted, or was there,
    /// and is taken to be where it was placed.
    Overdue,
}

impl Landed {
    /// How a window that reports where it is lands: `there`, where it was
    /// placed; elsewhere once it is no longer waited for by time, `late`;
    /// or elsewhere with nothing on its way.
    pub fn reported(there: bool, late: bool) -> Self {
        match (there, late) {
            (true, _) => Self::There,
            (false, true) => Self::Late,
            (false, false) => Self::Away,
        }
    }
}

/// The line of a window a transition placed behind what was on its way to
/// it, once it lands, with how long after the placement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LandedLine {
    /// The window.
    pub window: WindowId,
    /// How long after the placement was posted.
    pub after: Duration,
    /// How it was found to land.
    pub how: Landed,
}

impl fmt::Display for LandedLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let window = self.window;
        f.write_str("transition: ")?;
        match self.how {
            Landed::There => write!(f, "{window} is in place,"),
            Landed::Away => write!(
                f,
                "{window} is not where it was placed, with nothing on its way,"
            ),
            Landed::Late => write!(f, "{window} is taken to be in place as it moves at last,"),
            Landed::Overdue => write!(f, "{window} is taken to be in place, not reported there"),
        }?;
        let millis = self.after.as_millis();
        write!(f, " {millis} ms after its placement was posted")
    }
}

#[cfg(test)]
#[path = "tests/journal_lines_placed_tests.rs"]
mod tests;
