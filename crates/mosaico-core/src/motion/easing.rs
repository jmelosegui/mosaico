//! The curves a motion follows over time.

use std::f64::consts::LN_2;

use serde::{Deserialize, Serialize};

use super::CubicBezier;

/// What `1 - 2^(-10 t)` reaches at `t = 1`. `ease-out-expo` is divided by
/// it so that it ends at exactly 1 instead of jumping the last 0.1%.
const EXPO_END: f64 = 1.0 - 1.0 / 1024.0;

/// How a motion's progress follows the time it has run: a curve from time
/// progress 0 to 1 onto motion progress 0 to 1.
///
/// The presets that ease out start fast and slow down into the target,
/// which reads as responsive. [`CubicBezier`] takes a curve of the
/// user's own, written the way CSS writes it. No easing ever goes back or
/// past the target, so windows never overshoot.
///
/// `config.toml` writes an easing by name, which is how it is shown and
/// read back too: `"ease-out-cubic"`, or `"cubic-bezier(0.2, 0, 0, 1)"`.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum Easing {
    /// `linear`: the same speed all the way, stopping at once.
    Linear,
    /// `ease-out-quad`: `1 - (1 - t)^2`.
    EaseOutQuad,
    /// `ease-out-cubic`: `1 - (1 - t)^3`. The default.
    #[default]
    EaseOutCubic,
    /// `ease-in-out-cubic`: speeds up for the first half of the time and
    /// slows down for the second.
    EaseInOutCubic,
    /// `ease-out-expo`: `1 - 2^(-10 t)`, scaled to end at exactly 1. The
    /// fastest start of the presets.
    EaseOutExpo,
    /// `cubic-bezier(x1, y1, x2, y2)`: a curve of the user's own.
    CubicBezier(CubicBezier),
}

impl Easing {
    /// How far the motion has got after `t` of its time: 0 where it
    /// starts, 1 at its target.
    ///
    /// `t` is clamped to 0..=1, and not a number counts as 0. The result is
    /// exactly 0 at 0 and exactly 1 at 1, so a motion starts and ends
    /// exactly where it should, and in between it never goes back nor
    /// leaves 0..=1.
    pub fn progress(self, t: f64) -> f64 {
        let t = clamp_time(t);
        if t == 0.0 || t == 1.0 {
            return t;
        }
        match self {
            Self::Linear => t,
            Self::EaseOutQuad => 1.0 - (1.0 - t).powi(2),
            Self::EaseOutCubic => 1.0 - (1.0 - t).powi(3),
            Self::EaseInOutCubic if t < 0.5 => 4.0 * t.powi(3),
            Self::EaseInOutCubic => 1.0 - (2.0 - 2.0 * t).powi(3) / 2.0,
            Self::EaseOutExpo => (1.0 - (-10.0 * t).exp2()) / EXPO_END,
            Self::CubicBezier(curve) => curve.progress(t),
        }
    }

    /// The control points `[x1, y1, x2, y2]` of the cubic Bezier that
    /// looks most like the easing, to start a curve of one's own from. A
    /// cubic Bezier gives its own; `ease-out-expo` is approximated.
    pub fn control_points(self) -> [f64; 4] {
        match self {
            Self::Linear => [0.0, 0.0, 1.0, 1.0],
            Self::EaseOutQuad => [0.5, 1.0, 0.89, 1.0],
            Self::EaseOutCubic => [0.33, 1.0, 0.68, 1.0],
            Self::EaseInOutCubic => [0.65, 0.0, 0.35, 1.0],
            Self::EaseOutExpo => [0.16, 1.0, 0.3, 1.0],
            Self::CubicBezier(curve) => curve.points(),
        }
    }

    /// How fast [`progress`](Self::progress) changes at `t`: the slope of
    /// the curve there. A motion's speed is its distance times this,
    /// divided by its duration.
    ///
    /// `t` is clamped the same way. The slope is never negative. Where a
    /// cubic Bezier is vertical, as at the start of one whose x1 is 0, it
    /// is unbounded; it is capped at 20, so a speed is always a number.
    pub fn derivative(self, t: f64) -> f64 {
        let t = clamp_time(t);
        match self {
            Self::Linear => 1.0,
            Self::EaseOutQuad => 2.0 * (1.0 - t),
            Self::EaseOutCubic => 3.0 * (1.0 - t).powi(2),
            Self::EaseInOutCubic if t < 0.5 => 12.0 * t.powi(2),
            Self::EaseInOutCubic => 3.0 * (2.0 - 2.0 * t).powi(2),
            Self::EaseOutExpo => 10.0 * LN_2 * (-10.0 * t).exp2() / EXPO_END,
            Self::CubicBezier(curve) => curve.slope(t),
        }
    }
}

/// `t` within 0..=1, with not a number taken as 0.
fn clamp_time(t: f64) -> f64 {
    if t.is_nan() { 0.0 } else { t.clamp(0.0, 1.0) }
}

#[cfg(test)]
#[path = "tests/easing_tests.rs"]
mod tests;
