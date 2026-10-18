//! Where a split may go so that the windows on both sides keep their
//! minimum sizes.
//!
//! A split is a position along an axis, in the unit its layout works in:
//! how far the first window's side reaches for the BSP chain and the
//! master of the vertical stack, and how wide the centered master of the
//! three columns is. Its limits are the positions that leave each side its
//! minimum. A side's minimum is that of the whole region there: the
//! windows of a BSP chain beyond a split share what is left of it, so
//! they need the width or height their own splits ask for, and a stack
//! needs its widest window. When the limits cross, the area cannot hold
//! every minimum, and the split is left where it is.

use crate::LayoutKind;

use super::min_size::MinSize;

/// The least a region of the BSP chain needs along an axis: `mins` are its
/// windows, the first on one side of the region's own split, which runs
/// across `split_horizontal`, and the rest beyond it, split again across
/// the other way.
fn chain_need(mins: &[MinSize], split_horizontal: bool, horizontal: bool, gap: i32) -> i32 {
    let [first, rest @ ..] = mins else {
        return 0;
    };
    let first = first.along(horizontal);
    if rest.is_empty() {
        return first;
    }
    let rest = chain_need(rest, !split_horizontal, horizontal, gap);
    if split_horizontal != horizontal {
        first.max(rest)
    } else if first == 0 && rest == 0 {
        0
    } else {
        first + gap + rest
    }
}

/// The least width a stack of windows needs: that of its widest.
pub(super) fn widest(mins: &[MinSize]) -> i32 {
    mins.iter().map(|m| m.along(true)).max().unwrap_or(0)
}

/// The limits of a split `len` long that leaves `first` to one side and
/// `rest` to the other, with `gap` between them. `None` for no minimum.
pub(super) fn bounds(first: i32, rest: i32, gap: i32, len: i32) -> Option<(i32, i32)> {
    let half = gap / 2;
    (first > 0 || rest > 0).then(|| (first.max(1) + half, len - half - rest.max(1)))
}

/// The limits of a split of the BSP chain, across when `horizontal`, that
/// divides `mins`, the windows of the region it splits, into the first and
/// the rest.
pub(super) fn chain_limits(
    mins: &[MinSize],
    horizontal: bool,
    gap: i32,
    len: i32,
) -> Option<(i32, i32)> {
    let [first, rest @ ..] = mins else {
        return None;
    };
    let rest = chain_need(rest, !horizontal, horizontal, gap);
    bounds(first.along(horizontal), rest, gap, len)
}

/// The limits of the width of a centered master, in an area `len` wide
/// with a `gap` on each side of it, that leaves it `master` and each of
/// the columns beside it `side`.
pub(super) fn centered(master: i32, side: i32, gap: i32, len: i32) -> Option<(i32, i32)> {
    (master > 0 || side > 0).then(|| (master.max(1), len - gap * 2 - side.max(1) * 2))
}

/// The limits of split `split` of `kind`, whose area is `len` wide for the
/// splits across and `len` high for the others, for windows with minimums
/// `mins`, in the order of the layout. `None` where no window asks for
/// one. See [`fit`] for how a position is kept within them.
pub(super) fn limits(
    kind: LayoutKind,
    mins: &[MinSize],
    gap: i32,
    split: usize,
    len: i32,
) -> Option<(i32, i32)> {
    match kind {
        LayoutKind::Bsp => {
            let horizontal = split.is_multiple_of(2);
            let [first, rest @ ..] = mins.get(split..)? else {
                return None;
            };
            let rest = chain_need(rest, !horizontal, horizontal, gap);
            bounds(first.along(horizontal), rest, gap, len)
        }
        LayoutKind::ThreeColumn if mins.len() > 2 => {
            let (master, sides) = mins.split_first()?;
            centered(master.along(true), widest(sides), gap, len)
        }
        LayoutKind::VerticalStack | LayoutKind::ThreeColumn => {
            let (master, stack) = mins.split_first()?;
            bounds(master.along(true), widest(stack), gap, len)
        }
    }
}

/// `position` kept within `limits`, or as it is when there are none or
/// they cross.
pub(super) fn fit(position: i32, limits: Option<(i32, i32)>) -> i32 {
    match limits {
        Some((least, most)) if least <= most => position.clamp(least, most),
        _ => position,
    }
}

#[cfg(test)]
#[path = "min_limits_tests.rs"]
mod tests;
