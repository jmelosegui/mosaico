//! A workspace's windows as its layout has them, and the area each of its
//! splits divides.

use crate::{LayoutKind, Rect};

use super::SplitRatios;

/// A workspace's windows as its layout has them: what
/// [`resized_splits`](super::resized_splits) works out where dropped edges
/// put the splits from.
#[derive(Debug, Clone, Copy)]
pub struct Tiles<'a> {
    /// The layout.
    pub(super) kind: LayoutKind,
    /// The gap between windows.
    pub(super) gap: i32,
    /// The slot of each window, in window order.
    pub(super) slots: &'a [Rect],
    /// The splits, and the heights of the stacks, the slots are laid out
    /// with.
    pub(super) splits: &'a SplitRatios,
}

impl<'a> Tiles<'a> {
    /// The windows of `kind`, with `gap` between them, in `slots`, which
    /// `kind` lays out with `splits`.
    pub fn new(kind: LayoutKind, gap: i32, slots: &'a [Rect], splits: &'a SplitRatios) -> Self {
        Self {
            kind,
            gap,
            slots,
            splits,
        }
    }

    /// The area split `split` divides, from the slots of its windows:
    /// windows `split..` of the BSP chain, and every window for the
    /// master's split 0. The edges down a stack border no such split.
    pub(super) fn area_of(&self, split: usize) -> Option<Rect> {
        bounds(self.slots.get(split..)?)
    }
}

/// The smallest rect that holds all of `rects`.
fn bounds(rects: &[Rect]) -> Option<Rect> {
    let left = rects.iter().map(|r| r.x).min()?;
    let top = rects.iter().map(|r| r.y).min()?;
    let right = rects.iter().map(|r| r.x + r.width).max()?;
    let bottom = rects.iter().map(|r| r.y + r.height).max()?;
    Some(Rect::new(left, top, right - left, bottom - top))
}
