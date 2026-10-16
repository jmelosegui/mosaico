//! Timing curves written the way CSS writes `cubic-bezier()`.

/// How close to the time asked for the solver has to come. In a motion of
/// a second that is a tenth of a microsecond.
const EPSILON: f64 = 1e-7;

/// Newton steps tried before falling back to bisection.
const NEWTON_STEPS: usize = 8;

/// Bisection steps at most, which narrow the curve's parameter down to
/// 2^-40 of its range: well inside [`EPSILON`] wherever time moves.
const BISECTION_STEPS: usize = 40;

/// How slowly time has to move along the curve for its slope to be taken
/// from a short secant instead of from the derivatives, whose ratio is
/// unreliable or undefined there.
const FLAT: f64 = 1e-6;

/// Half the stretch of the curve's parameter that secant spans.
const SECANT: f64 = 1e-4;

/// The steepest slope a curve is taken to have.
const MAX_SLOPE: f64 = 20.0;

/// A timing curve from (0, 0) to (1, 1), bent towards the control points
/// (x1, y1) and (x2, y2), with the meaning CSS gives `cubic-bezier()`.
///
/// x is time and y progress, and all four are within 0..=1. Time then only
/// moves forward along the curve, and the progress never goes back nor
/// past 1, so a window never overshoots its target. CSS allows y outside
/// 0..=1 for curves that do; mosaico does not.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CubicBezier {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
}

impl CubicBezier {
    /// The curve with control points (x1, y1) and (x2, y2), or `None`
    /// unless all four are within 0..=1.
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> Option<Self> {
        let within = [x1, y1, x2, y2].iter().all(|v| (0.0..=1.0).contains(v));
        within.then_some(Self { x1, y1, x2, y2 })
    }

    /// The control points, as `[x1, y1, x2, y2]`.
    pub fn points(self) -> [f64; 4] {
        [self.x1, self.y1, self.x2, self.y2]
    }

    /// The progress at time `t`, for `t` within 0..=1.
    pub(super) fn progress(self, t: f64) -> f64 {
        coordinate(self.y1, self.y2, self.solve(t))
    }

    /// The slope at time `t`, for `t` within 0..=1: never negative, and
    /// capped at [`MAX_SLOPE`].
    pub(super) fn slope(self, t: f64) -> f64 {
        let u = self.solve(t);
        let dx = rate(self.x1, self.x2, u);
        let slope = if dx > FLAT {
            rate(self.y1, self.y2, u) / dx
        } else {
            // Time hardly moves here: the curve is vertical, or both
            // derivatives vanish. Time always moves forward along the
            // curve, so a secant over a short stretch has a slope, and it
            // tends to the curve's own as the stretch shrinks.
            let (a, b) = ((u - SECANT).max(0.0), (u + SECANT).min(1.0));
            let rise = coordinate(self.y1, self.y2, b) - coordinate(self.y1, self.y2, a);
            rise / (coordinate(self.x1, self.x2, b) - coordinate(self.x1, self.x2, a))
        };
        // The curve never goes back, so a slope below 0 is rounding.
        slope.clamp(0.0, MAX_SLOPE)
    }

    /// The parameter where the curve reaches time `t`, within
    /// [`EPSILON`].
    fn solve(self, t: f64) -> f64 {
        // Newton's method gets there in a few steps wherever the curve is
        // not close to vertical. Where it is, a step can overshoot out of
        // the curve, and bisection takes over.
        let mut u = t;
        for _ in 0..NEWTON_STEPS {
            let error = coordinate(self.x1, self.x2, u) - t;
            if error.abs() < EPSILON {
                return u;
            }
            let dx = rate(self.x1, self.x2, u);
            if dx < FLAT {
                break;
            }
            u -= error / dx;
            if !(0.0..=1.0).contains(&u) {
                break;
            }
        }
        // Time never goes back along the curve, so bisection always finds
        // the one parameter that reaches `t`.
        let (mut low, mut high) = (0.0, 1.0);
        let mut u = t;
        for _ in 0..BISECTION_STEPS {
            let x = coordinate(self.x1, self.x2, u);
            if (x - t).abs() < EPSILON {
                break;
            }
            if x < t {
                low = u;
            } else {
                high = u;
            }
            u = (low + high) / 2.0;
        }
        u
    }
}

/// One coordinate of the curve at parameter `u`, where the control points
/// have `p1` and `p2` for it and the ends 0 and 1.
fn coordinate(p1: f64, p2: f64, u: f64) -> f64 {
    let v = 1.0 - u;
    3.0 * v * v * u * p1 + 3.0 * v * u * u * p2 + u * u * u
}

/// How fast that coordinate changes with the parameter at `u`.
fn rate(p1: f64, p2: f64, u: f64) -> f64 {
    let v = 1.0 - u;
    3.0 * v * v * p1 + 6.0 * v * u * (p2 - p1) + 3.0 * u * u * (1.0 - p2)
}

#[cfg(test)]
#[path = "tests/bezier_tests.rs"]
mod tests;
