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
//! master stays centered, so both its edges follow either one. Down a stack
//! or a side column, a window borders the split above it with its top edge,
//! unless it is the first there, and its own with its bottom edge, unless it
//! is the last. Every other edge, an outer edge of the area, borders none.

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
    /// Down a stack or a column, with its bottom edge: it is the split's
    /// first window, above the split.
    Above,
    /// Down a stack or a column, with its top edge: it is the window below
    /// the split's first, a whole gap below it.
    Below,
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
        LayoutKind::ThreeColumn if count > 2 => columns_border(count, index, side),
        LayoutKind::VerticalStack | LayoutKind::ThreeColumn => stack_border(count, index, side),
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

/// The split the `side` edge of window `index` of `count` borders in the
/// vertical stack, or in two windows of the three columns, and how: the
/// master's, or one down the stack, whose first window is window `k` for
/// split `k`.
fn stack_border(count: usize, index: usize, side: Side) -> Option<(usize, Border)> {
    match (index, side) {
        (0, Side::Right) => Some((0, Border::First)),
        (1.., Side::Left) => Some((0, Border::Rest)),
        (2.., Side::Top) => Some((index - 1, Border::Below)),
        (1.., Side::Bottom) if index + 1 < count => Some((index, Border::Above)),
        _ => None,
    }
}

/// The split the `side` edge of window `index` of three or more in three
/// columns borders, and how: the master's split 0, or one down the column
/// the window is in, whose first window is window `k` for split `k`. The
/// windows after the master alternate between the left column and the
/// right one, so the window above another in its column is two before it.
fn columns_border(count: usize, index: usize, side: Side) -> Option<(usize, Border)> {
    let left_column = !index.is_multiple_of(2);
    let border = match (index, side) {
        (0, Side::Left) => (0, Border::MasterLeft),
        (0, Side::Right) => (0, Border::MasterRight),
        (_, Side::Right) if left_column => (0, Border::LeftColumn),
        (1.., Side::Left) if !left_column => (0, Border::RightColumn),
        (3.., Side::Top) => (index - 2, Border::Below),
        (1.., Side::Bottom) if index + 2 < count => (index, Border::Above),
        _ => return None,
    };
    Some(border)
}

#[cfg(test)]
#[path = "split_edges_tests.rs"]
mod tests;
