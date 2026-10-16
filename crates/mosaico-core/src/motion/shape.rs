//! The two curves a segment can follow from one rect to another.

use super::{Easing, Velocity};

/// How a segment gets from one rect to the other.
///
/// Both work on the rects' edges, `[left, top, right, bottom]` in
/// fractions of a pixel, at `s`, the fraction of the duration that has
/// passed, for a duration of `seconds`. Neither ever takes an edge back
/// or past where it goes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Shape {
    /// Starting at rest and following the easing.
    Eased(Easing),
    /// Leaving with this velocity and coming to rest at the target, along
    /// a cubic Hermite curve.
    Hermite(Velocity),
}

impl Shape {
    /// A Hermite curve from `from` to `to` that leaves with `velocity`, as
    /// much of it as keeps every edge from passing its target.
    ///
    /// Such a curve, ending at rest, stays between its ends as long as an
    /// edge leaves towards its end at no more than three times its average
    /// speed, which is how fast `ease-out-cubic` starts. An edge moving
    /// faster leaves at that speed. One moving away from its end, or whose
    /// speed is not a finite number, leaves at rest.
    pub(super) fn hermite(from: [f64; 4], to: [f64; 4], velocity: Velocity, seconds: f64) -> Self {
        let velocity = velocity.edges();
        let limited = std::array::from_fn(|i| {
            if seconds == 0.0 || !velocity[i].is_finite() {
                return 0.0;
            }
            let fastest = 3.0 * (to[i] - from[i]) / seconds;
            velocity[i].clamp(fastest.min(0.0), fastest.max(0.0))
        });
        Self::Hermite(Velocity::from_edges(limited))
    }

    /// Where the edges of a rect moving from `from` to `to` are at `s`.
    pub(super) fn position(self, from: [f64; 4], to: [f64; 4], s: f64, seconds: f64) -> [f64; 4] {
        match self {
            Self::Eased(easing) => {
                let progress = easing.progress(s);
                std::array::from_fn(|i| from[i] + (to[i] - from[i]) * progress)
            }
            Self::Hermite(initial) => {
                // The Hermite basis: how much of the way to the target the
                // edges have come, and how far their initial velocity has
                // carried them.
                let reach = s * s * (3.0 - 2.0 * s);
                let carry = s * (1.0 - s) * (1.0 - s) * seconds;
                let initial = initial.edges();
                std::array::from_fn(|i| from[i] + (to[i] - from[i]) * reach + initial[i] * carry)
            }
        }
    }

    /// How fast those edges move at `s`. An eased rect is at rest at its
    /// start, before it has moved.
    pub(super) fn velocity(self, from: [f64; 4], to: [f64; 4], s: f64, seconds: f64) -> Velocity {
        let velocity = match self {
            Self::Eased(_) if s == 0.0 => return Velocity::ZERO,
            Self::Eased(easing) => {
                let rate = easing.derivative(s) / seconds;
                std::array::from_fn(|i| (to[i] - from[i]) * rate)
            }
            Self::Hermite(initial) => {
                // How fast the two parts of `position` change.
                let reach = 6.0 * s * (1.0 - s) / seconds;
                let carry = (1.0 - s) * (1.0 - 3.0 * s);
                let initial = initial.edges();
                std::array::from_fn(|i| (to[i] - from[i]) * reach + initial[i] * carry)
            }
        };
        Velocity::from_edges(velocity)
    }
}
