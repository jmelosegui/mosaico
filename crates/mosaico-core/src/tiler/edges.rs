//! How far something reaches in from each edge of a monitor.

use crate::Rect;

/// An edge of a monitor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    /// The top edge.
    Top,
    /// The bottom edge.
    Bottom,
    /// The left edge.
    Left,
    /// The right edge.
    Right,
}

impl Side {
    /// Every edge, in the order config files list them.
    pub const ALL: [Self; 4] = [Self::Top, Self::Bottom, Self::Left, Self::Right];

    /// The name config files and messages give it.
    pub fn name(self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Bottom => "bottom",
            Self::Left => "left",
            Self::Right => "right",
        }
    }
}

/// How far something reaches in from each edge of a monitor, in pixels.
/// Never negative.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Edges {
    top: i32,
    bottom: i32,
    left: i32,
    right: i32,
}

impl Edges {
    /// Reaches `top`, `bottom`, `left` and `right` in from those edges.
    /// Negative counts as 0.
    pub fn new(top: i32, bottom: i32, left: i32, right: i32) -> Self {
        Self {
            top: top.max(0),
            bottom: bottom.max(0),
            left: left.max(0),
            right: right.max(0),
        }
    }

    /// What the system keeps on each edge of a monitor covering `bounds`
    /// when it leaves `work_area` for windows.
    pub fn kept_by(bounds: &Rect, work_area: &Rect) -> Self {
        Self::new(
            work_area.y - bounds.y,
            (bounds.y + bounds.height) - (work_area.y + work_area.height),
            work_area.x - bounds.x,
            (bounds.x + bounds.width) - (work_area.x + work_area.width),
        )
    }

    /// How far it reaches in from `side`.
    pub fn get(&self, side: Side) -> i32 {
        match side {
            Side::Top => self.top,
            Side::Bottom => self.bottom,
            Side::Left => self.left,
            Side::Right => self.right,
        }
    }

    /// Has it reach `pixels` in from `side`. Negative counts as 0.
    pub fn set(&mut self, side: Side, pixels: i32) {
        let pixels = pixels.max(0);
        match side {
            Side::Top => self.top = pixels,
            Side::Bottom => self.bottom = pixels,
            Side::Left => self.left = pixels,
            Side::Right => self.right = pixels,
        }
    }

    /// Whether it reaches in from no edge.
    pub fn is_none(&self) -> bool {
        *self == Self::default()
    }

    /// On each edge, whichever of the two reaches further.
    pub fn furthest(self, other: Self) -> Self {
        Self::new(
            self.top.max(other.top),
            self.bottom.max(other.bottom),
            self.left.max(other.left),
            self.right.max(other.right),
        )
    }

    /// Multiplied by a monitor's `scale`, rounded, for values given at
    /// 100%.
    pub fn scaled(self, scale: f64) -> Self {
        let scale = if scale.is_finite() && scale > 0.0 {
            scale
        } else {
            1.0
        };
        let by = |pixels: i32| (f64::from(pixels) * scale).round() as i32;
        Self::new(by(self.top), by(self.bottom), by(self.left), by(self.right))
    }

    /// The edges on which it reaches past a quarter of a monitor covering
    /// `bounds`, which is refused, so nothing can take a monitor away.
    pub fn too_large(&self, bounds: &Rect) -> Vec<Side> {
        let limit = |side: Side| match side {
            Side::Top | Side::Bottom => bounds.height / 4,
            Side::Left | Side::Right => bounds.width / 4,
        };
        (Side::ALL.into_iter())
            .filter(|&side| self.get(side) > limit(side))
            .collect()
    }

    /// Itself with every edge [`too_large`](Self::too_large) for `bounds`
    /// left out.
    pub fn within(mut self, bounds: &Rect) -> Self {
        for side in self.too_large(bounds) {
            self.set(side, 0);
        }
        self
    }
}

#[cfg(test)]
#[path = "tests/edges_tests.rs"]
mod tests;
