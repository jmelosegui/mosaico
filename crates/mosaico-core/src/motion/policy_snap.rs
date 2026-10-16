//! Why a placement snaps into place instead of animating.
//!
//! These are the rules of [`step_animates`](super::step_animates) and
//! [`still_animates`](super::still_animates), each with the reason it
//! gives when it does not hold, so the log can say why a window snapped.

use std::fmt;

use super::{ApplyFacts, MAX_ANIMATED_WINDOWS, MIN_TRAVEL, StepFacts, travel};

/// Why the tiler does not let a placement animate as it finishes a step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepSnap {
    /// Transitions are off, take no time, or the session has no frames.
    Off,
    /// The step lays windows out from scratch: a display change, the
    /// tiler starting, or the bar reservation right after it.
    Rebuild,
    /// The step moves this many windows, more than
    /// [`MAX_ANIMATED_WINDOWS`].
    TooManyWindows(usize),
    /// The window appears in the step.
    Appears,
    /// The window waits behind the window in monocle.
    BehindMonocle,
    /// The window moves to another monitor.
    OtherMonitor,
}

/// Why the tiler keeps a placement with `facts` from animating, or `None`
/// when it lets it animate. The reasons about the whole step come first.
pub fn step_snap(facts: &StepFacts) -> Option<StepSnap> {
    let why = if !facts.enabled {
        StepSnap::Off
    } else if facts.rebuild {
        StepSnap::Rebuild
    } else if facts.windows_moved > MAX_ANIMATED_WINDOWS {
        StepSnap::TooManyWindows(facts.windows_moved)
    } else if facts.appears {
        StepSnap::Appears
    } else if facts.behind_monocle {
        StepSnap::BehindMonocle
    } else if !facts.same_monitor {
        StepSnap::OtherMonitor
    } else {
        return None;
    };
    Some(why)
}

impl fmt::Display for StepSnap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Off => f.write_str("transitions are off"),
            Self::Rebuild => f.write_str("the step lays windows out from scratch"),
            Self::TooManyWindows(n) => {
                write!(
                    f,
                    "the step moves {n} windows, more than {MAX_ANIMATED_WINDOWS}"
                )
            }
            Self::Appears => f.write_str("it appears in this step"),
            Self::BehindMonocle => f.write_str("it waits behind the monocle window"),
            Self::OtherMonitor => f.write_str("it moves to another monitor"),
        }
    }
}

/// Why a placement the tiler let animate snaps as it is applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplySnap {
    /// A display change is settling: the layout is about to be rebuilt.
    /// The applier finds this first, and then asks nothing of the window,
    /// so [`apply_snap`] never gives it.
    DisplaysChanging,
    /// The system's setting turns animations off, and the config follows
    /// it.
    SystemOff,
    /// The window is minimized.
    Minimized,
    /// The window is maximized.
    Maximized,
    /// Transitions are off for the session: the platform cannot move
    /// windows frame by frame, or the daemon has no display frames to move
    /// them on. The tiler marks every placement to snap from then on, so
    /// only one it held back since before gets this far.
    NoFrames,
    /// No edge of the window travels more than [`MIN_TRAVEL`] pixels: this
    /// far at most.
    ShortTravel(i64),
}

/// Why a placement with `facts` snaps as it is applied, or `None` when it
/// still animates.
pub fn apply_snap(facts: &ApplyFacts) -> Option<ApplySnap> {
    let distance = travel(facts.from, facts.to);
    let why = if facts.follow_system && !facts.system_allows {
        ApplySnap::SystemOff
    } else if facts.minimized {
        ApplySnap::Minimized
    } else if facts.maximized {
        ApplySnap::Maximized
    } else if !facts.frames_supported {
        ApplySnap::NoFrames
    } else if distance <= i64::from(MIN_TRAVEL) {
        ApplySnap::ShortTravel(distance)
    } else {
        return None;
    };
    Some(why)
}

impl fmt::Display for ApplySnap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DisplaysChanging => f.write_str("the displays are changing"),
            Self::SystemOff => f.write_str("the system turns animations off"),
            Self::Minimized => f.write_str("it is minimized"),
            Self::Maximized => f.write_str("it is maximized"),
            Self::NoFrames => f.write_str("transitions are off for this session"),
            Self::ShortTravel(px) => write!(f, "it travels {px} px, {MIN_TRAVEL} or less"),
        }
    }
}

#[cfg(test)]
#[path = "tests/policy_snap_tests.rs"]
mod tests;
