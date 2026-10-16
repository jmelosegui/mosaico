//! Window transitions: how a window gets from one rect to another over
//! time.
//!
//! Everything here is pure and works on the time it is given, so a
//! transition can be checked at any instant without waiting for it.
//! [`Easing`](crate::motion::Easing) is the curve a motion follows.

mod bezier;
mod easing;
mod easing_names;

pub use bezier::CubicBezier;
pub use easing::Easing;
