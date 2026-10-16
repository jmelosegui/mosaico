//! How fast a moving rect changes.

/// How fast each edge of a moving rect moves, in pixels per second:
/// positive to the right for the left and right edges, and down for the
/// top and bottom ones.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Velocity {
    /// How fast the left edge moves.
    pub left: f64,
    /// How fast the top edge moves.
    pub top: f64,
    /// How fast the right edge moves.
    pub right: f64,
    /// How fast the bottom edge moves.
    pub bottom: f64,
}

impl Velocity {
    /// Standing still.
    pub const ZERO: Self = Self {
        left: 0.0,
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
    };

    /// The edges' speeds as `[left, top, right, bottom]`, the order
    /// [`Velocity::from_edges`] takes them in.
    pub(super) fn edges(self) -> [f64; 4] {
        [self.left, self.top, self.right, self.bottom]
    }

    /// The velocity with these speeds, as `[left, top, right, bottom]`.
    pub(super) fn from_edges([left, top, right, bottom]: [f64; 4]) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }
}
