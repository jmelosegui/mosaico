//! Window transitions: how a window gets from one rect to another over
//! time.
//!
//! Everything here is pure and works on the time it is given, so a
//! transition can be checked at any instant without waiting for it.
//! [`Easing`](crate::motion::Easing) is the curve a motion follows,
//! [`Segment`](crate::motion::Segment) one rect moving along it, and
//! [`Motions`](crate::motion::Motions) the table of moving windows that
//! each display frame asks where the windows are.

mod bezier;
mod easing;
mod easing_names;
mod motions;
mod segment;
mod shape;
mod spec;
mod velocity;

#[cfg(test)]
#[path = "tests/support.rs"]
mod test_support;

pub use bezier::CubicBezier;
pub use easing::Easing;
pub use motions::{Motions, Tick};
pub use segment::Segment;
pub use spec::MotionSpec;
pub use velocity::Velocity;
