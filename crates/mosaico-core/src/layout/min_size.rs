//! The smallest size a window can have.

/// The smallest size a window's application lets it have, as visible
/// bounds in the units of [`Rect`](crate::Rect).
///
/// A side of 0 means the window has no minimum that way. Layouts keep a
/// window's slot at least this big where the area has room for every
/// window's minimum, see [`SplitRatios::apply_fitted`](super::SplitRatios::apply_fitted).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MinSize {
    /// The least width, or 0 for none.
    pub width: i32,
    /// The least height, or 0 for none.
    pub height: i32,
}

impl MinSize {
    /// A minimum of `width` by `height`.
    pub fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }

    /// Whether the window has no minimum either way.
    pub fn is_none(self) -> bool {
        self.width <= 0 && self.height <= 0
    }

    /// The least width when `horizontal`, and the least height otherwise.
    pub(super) fn along(self, horizontal: bool) -> i32 {
        if horizontal { self.width } else { self.height }.max(0)
    }

    /// The minimum of a window that reports `outer`, the least size of
    /// the whole of it, with `margin` of that not part of its visible
    /// bounds. A side of `outer` at or below the same side of `floor`,
    /// the least every window has, is no minimum.
    pub fn within(outer: MinSize, floor: MinSize, margin: MinSize) -> MinSize {
        let side = |outer: i32, floor: i32, margin: i32| {
            if outer <= floor {
                0
            } else {
                (outer - margin).max(1)
            }
        };
        MinSize::new(
            side(outer.width, floor.width, margin.width),
            side(outer.height, floor.height, margin.height),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::MinSize;

    #[test]
    fn a_side_at_the_floor_is_no_minimum() {
        let found = MinSize::within(
            MinSize::new(136, 1000),
            MinSize::new(136, 39),
            MinSize::new(14, 7),
        );
        assert_eq!(found, MinSize::new(0, 993));
    }

    #[test]
    fn the_margin_is_not_part_of_the_visible_minimum() {
        let found = MinSize::within(
            MinSize::new(914, 500),
            MinSize::new(136, 39),
            MinSize::new(14, 7),
        );
        assert_eq!(found, MinSize::new(900, 493));
    }

    #[test]
    fn a_margin_never_takes_the_minimum_to_nothing() {
        let found = MinSize::within(
            MinSize::new(50, 50),
            MinSize::new(10, 10),
            MinSize::new(80, 80),
        );
        assert_eq!(found, MinSize::new(1, 1));
    }
}
