//! The lines the log gets as a step sets windows moving, and as
//! placements snap.

use std::fmt;

use crate::WindowId;

use super::super::{MotionSpec, StepSnap};

/// The line of a step that gives windows a target to move to.
#[derive(Debug, Clone, PartialEq)]
pub struct StepLine {
    /// The windows that start moving, in the order they were given their
    /// targets. They move along the spec's easing.
    pub started: Vec<WindowId>,
    /// The windows already moving that turn towards a new target, in the
    /// same order. Each keeps as much of its speed as it can, so it
    /// follows a curve of its own rather than the easing.
    pub turned: Vec<WindowId>,
    /// The duration and easing of their motions.
    pub spec: MotionSpec,
}

impl fmt::Display for StepLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let millis = self.spec.duration.as_millis();
        f.write_str("transition:")?;
        if !self.started.is_empty() {
            let count = self.started.len();
            write!(f, " {count} window(s) move over {millis} ms")?;
            write!(f, " with {}:", self.spec.easing)?;
            windows(f, &self.started)?;
        }
        if !self.turned.is_empty() {
            let count = self.turned.len();
            let after = if self.started.is_empty() { "" } else { ";" };
            write!(
                f,
                "{after} {count} window(s) turn to new targets over {millis} ms:"
            )?;
            windows(f, &self.turned)?;
        }
        Ok(())
    }
}

/// Writes `windows`, each after a space, separated by commas.
fn windows(f: &mut fmt::Formatter<'_>, windows: &[WindowId]) -> fmt::Result {
    for (i, window) in windows.iter().enumerate() {
        let comma = if i == 0 { "" } else { "," };
        write!(f, "{comma} {window}")?;
    }
    Ok(())
}

/// The line of a placement that snaps into place, and why.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnapLine<Why> {
    /// The window placed.
    pub window: WindowId,
    /// Why it snaps.
    pub why: Why,
}

impl<Why: fmt::Display> fmt::Display for SnapLine<Why> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "transition: {} snaps: {}", self.window, self.why)
    }
}

/// The line of a step all of whose placements snap, for a reason about
/// the step as a whole.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StepSnapLine {
    /// How many placements snap.
    pub placements: usize,
    /// Why they do.
    pub why: StepSnap,
}

impl fmt::Display for StepSnapLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let count = self.placements;
        write!(f, "transition: {count} placement(s) snap: {}", self.why)
    }
}

#[cfg(test)]
#[path = "tests/journal_lines_tests.rs"]
mod tests;
