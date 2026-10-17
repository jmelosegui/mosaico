//! Which split an edge of a tile borders.
//!
//! Split `k` divides windows `k..` into window `k` and the rest, side by
//! side for even `k` in BSP and one above the other for odd `k`. There,
//! window `k` borders split `k` with its far edge, its right one for even
//! `k` and its bottom one for odd `k`, unless it is the last window, and
//! splits `k - 1` and `k - 2`, which it lies beyond, with its near edges.
//! In the vertical stack, the master's right edge and the left edge of each
//! window of the stack border split 0, and two windows in the three columns
//! split the same way. With three or more there, the master's two edges and
//! the inner edge of each window of a side column border split 0, and the
//! master stays centered, so both its edges follow either one. Every other
//! edge, as an outer edge or one between two windows of a stack or of a
//! side column, borders none.

use crate::LayoutKind;

/// An edge of a window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Side {
    Left,
    Top,
    Right,
    Bottom,
}

/// How a window borders a split, which says what an edge of it dropped
/// somewhere makes of the split.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Border {
    /// With its far edge: it is the split's first window.
    First,
    /// With a near edge: it lies in the rest, beyond the split.
    Rest,
    /// The centered master of the three columns, with its left edge.
    MasterLeft,
    /// The centered master of the three columns, with its right edge.
    MasterRight,
    /// A window of the left column, with its right edge.
    LeftColumn,
    /// A window of the right column, with its left edge.
    RightColumn,
}

/// The split the `side` edge of window `index` of `count` borders in
/// `kind`, and how, if it borders one.
pub(super) fn border(
    kind: LayoutKind,
    count: usize,
    index: usize,
    side: Side,
) -> Option<(usize, Border)> {
    if index >= count || count < 2 {
        return None;
    }
    match kind {
        LayoutKind::Bsp => chain_border(count, index, side),
        LayoutKind::ThreeColumn if count > 2 => columns_border(index, side),
        LayoutKind::VerticalStack | LayoutKind::ThreeColumn => match (index, side) {
            (0, Side::Right) => Some((0, Border::First)),
            (1.., Side::Left) => Some((0, Border::Rest)),
            _ => None,
        },
    }
}

/// The split of the BSP chain the `side` edge of window `index` of `count`
/// borders, and how.
fn chain_border(count: usize, index: usize, side: Side) -> Option<(usize, Border)> {
    let across = |split: usize| split.is_multiple_of(2);
    let far = if across(index) {
        Side::Right
    } else {
        Side::Bottom
    };
    if side == far && index + 1 < count {
        return Some((index, Border::First));
    }
    let near = |split: usize| if across(split) { Side::Left } else { Side::Top };
    (index.saturating_sub(2)..index)
        .find(|&split| near(split) == side)
        .map(|split| (split, Border::Rest))
}

/// How the `side` edge of window `index` of three or more in three
/// columns borders split 0, if it does. The windows after the master
/// alternate between the left column and the right one.
fn columns_border(index: usize, side: Side) -> Option<(usize, Border)> {
    let left_column = !index.is_multiple_of(2);
    let border = match (index, side) {
        (0, Side::Left) => Border::MasterLeft,
        (0, Side::Right) => Border::MasterRight,
        (_, Side::Right) if left_column => Border::LeftColumn,
        (1.., Side::Left) if !left_column => Border::RightColumn,
        _ => return None,
    };
    Some((0, border))
}

#[cfg(test)]
#[path = "split_edges_tests.rs"]
mod tests;
