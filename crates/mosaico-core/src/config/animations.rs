//! Window transitions: whether windows move to their new place, and how.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::motion::{Easing, MotionSpec};

use super::limits::DURATION_MAX_MS;

/// Window transitions: the `[animations]` section of `config.toml`.
///
/// When they are on, windows that swap, move or resize, and the windows a
/// layout change moves, go to their new place over a short time instead
/// of jumping there.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AnimationConfig {
    /// Whether windows move to their new place in a transition. On by
    /// default; with `follow_system`, windows still snap while the
    /// system's own setting turns animations off.
    pub enabled: bool,
    /// How long a transition lasts, in milliseconds, from 0 to 1000.
    pub duration: i32,
    /// The curve windows follow. One that is not a preset or a cubic
    /// Bezier mosaico takes makes the file invalid, as any other value
    /// out of its choices does.
    pub easing: Easing,
    /// Whether windows snap into place while the system's own setting
    /// turns animations off.
    pub follow_system: bool,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            duration: 180,
            easing: Easing::default(),
            follow_system: true,
        }
    }
}

impl AnimationConfig {
    /// Whether windows move to their new place in transitions: enabled,
    /// with a duration above zero, as a transition that takes no time is a
    /// placement made at once. This is what the step decision's
    /// [`StepFacts::enabled`](crate::motion::StepFacts::enabled) stands
    /// for.
    pub fn is_on(&self) -> bool {
        self.enabled && self.duration > 0
    }

    /// The duration and easing of a transition that starts now. The
    /// duration is within 0 to 1000 ms even in a config that was not
    /// validated.
    pub fn spec(&self) -> MotionSpec {
        let millis = self.duration.clamp(0, DURATION_MAX_MS).unsigned_abs();
        MotionSpec {
            duration: Duration::from_millis(millis.into()),
            easing: self.easing,
        }
    }
}

#[cfg(test)]
#[path = "animations_tests.rs"]
mod tests;
