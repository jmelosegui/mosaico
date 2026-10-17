//! What a resize changed in the splits of a layout, as the tiler logs it:
//! each split it moved, with its ratio, and where each window but the last
//! ends in each stack it gave new heights.

use std::fmt;

use crate::LayoutKind;

use super::SplitRatios;

/// The splits and the stacks of a layout that a resize changed.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Changes {
    /// Each split moved, with its ratio.
    splits: Vec<(usize, f64)>,
    /// The ends of each stack, or side column, given new heights.
    ends: Vec<Vec<f64>>,
}

/// What `after` changed from `before` in `kind`, for `count` windows.
pub(crate) fn changes(
    before: &SplitRatios,
    after: &SplitRatios,
    kind: LayoutKind,
    count: usize,
) -> Changes {
    let moved = |k: usize| {
        let ratio = after.ratio(kind, k)?;
        (before.ratio(kind, k) != Some(ratio)).then_some((k, ratio))
    };
    let columns: &[usize] = match kind {
        LayoutKind::ThreeColumn => &[1, 2],
        _ => &[1],
    };
    let ends = |&window: &usize| {
        let ends = after.column(kind, window)?.ends()?;
        let was = before.column(kind, window).and_then(|column| column.ends());
        (was != Some(ends)).then(|| ends.to_vec())
    };
    Changes {
        splits: (0..count).filter_map(moved).collect(),
        ends: columns.iter().filter_map(ends).collect(),
    }
}

impl fmt::Display for Changes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.splits)?;
        self.ends
            .iter()
            .try_for_each(|ends| write!(f, ", ends {ends:?}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::layout::split_support::unmoved;

    #[test]
    fn a_resize_shows_the_splits_it_moved_and_the_ends_of_the_stacks_it_resized() {
        // Arrange -- three windows, the stack resized, then its master moved,
        // and its heights changed too; and the first split of BSP moved,
        // where the second was moved before.
        let (kind, mut before) = (LayoutKind::VerticalStack, unmoved(3));
        before.set_heights(kind, 1, &[0.25, 0.75]);
        before.set(LayoutKind::Bsp, 1, 0.3);
        let mut master = before.clone();
        master.set(kind, 0, 0.6);
        let mut both = master.clone();
        both.set_heights(kind, 1, &[0.5, 0.5]);
        let mut bsp = before.clone();
        bsp.set(LayoutKind::Bsp, 0, 0.6);

        // Act
        let shown = [
            changes(&before, &master, kind, 3).to_string(),
            changes(&before, &both, kind, 3).to_string(),
            changes(&before, &bsp, LayoutKind::Bsp, 3).to_string(),
            changes(&before, &before, LayoutKind::ThreeColumn, 3).to_string(),
        ];

        // Assert
        let expected = ["[(0, 0.6)]", "[(0, 0.6)], ends [0.5]", "[(0, 0.6)]", "[]"];
        assert_eq!(shown, expected);
    }
}
