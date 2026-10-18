//! What the tests of the splits share: layouts laid out, and edges dragged.

use crate::event::Edges;
use crate::id::wid;
use crate::{LayoutKind, Rect, WindowId};

use super::split_edges::{Border, Side, border};
use super::{SplitRatios, Tiles, resized_splits};

/// The area every layout of these tests fills.
pub(super) const AREA: Rect = Rect {
    x: 0,
    y: 0,
    width: 1920,
    height: 1080,
};

/// The gap between windows in these tests.
pub(super) const GAP: i32 = 8;

/// Every edge, in the order [`Edges`] has them.
pub(super) const SIDES: [Side; 4] = [Side::Left, Side::Top, Side::Right, Side::Bottom];

/// Every layout.
pub(super) const KINDS: [LayoutKind; 3] = [
    LayoutKind::Bsp,
    LayoutKind::VerticalStack,
    LayoutKind::ThreeColumn,
];

/// The splits of `count` windows, none of them moved.
pub(super) fn unmoved(count: usize) -> SplitRatios {
    let mut ratios = SplitRatios::default();
    for index in 0..count {
        ratios.insert(index, index);
    }
    ratios
}

/// Where `ratios` lay out `count` windows in `kind`, in window order.
pub(super) fn slots(kind: LayoutKind, count: usize, ratios: &SplitRatios) -> Vec<Rect> {
    let windows: Vec<WindowId> = (1..=count as u64).map(wid).collect();
    let laid_out = ratios.apply(kind, GAP, 0.5, &windows, &AREA);
    let slot = |window: &WindowId| laid_out.iter().find(|(w, _)| w == window).map(|w| w.1);
    windows.iter().filter_map(slot).collect()
}

/// Where `count` windows of `kind` are with the splits `moved`, or with
/// the splits nobody moved when it is `None`, in window order.
pub(super) fn after(kind: LayoutKind, count: usize, moved: &Option<SplitRatios>) -> Vec<Rect> {
    let fresh = unmoved(count);
    slots(kind, count, moved.as_ref().unwrap_or(&fresh))
}

/// The splits of `kind` that `moved` has a ratio for, among those of
/// `count` windows, as nobody moved any before.
pub(super) fn moved_splits(
    kind: LayoutKind,
    count: usize,
    moved: &Option<SplitRatios>,
) -> Vec<usize> {
    let has = |k: &usize| moved.as_ref().is_some_and(|m| m.ratio(kind, *k).is_some());
    (0..count).filter(has).collect()
}

/// What the user dropping the `edges` of window `index` of `slots`, laid
/// out with `kind` and splits nobody moved, at `rect` does to the splits,
/// once grabbed at `start`.
pub(super) fn dropped(
    kind: LayoutKind,
    slots: &[Rect],
    index: usize,
    edges: Edges,
    start: Rect,
    rect: Rect,
) -> Option<SplitRatios> {
    let ratios = unmoved(slots.len());
    resized_splits(
        Tiles::new(kind, GAP, slots, &ratios),
        index,
        edges,
        start,
        rect,
    )
}

/// The two windows beside the edge that the `side` edge of window `index`
/// of `count` in `kind` is on, when that edge is one down a stack.
pub(super) fn beside(
    kind: LayoutKind,
    count: usize,
    index: usize,
    side: Side,
) -> Option<[usize; 2]> {
    match border(kind, count, index, side)? {
        (split, Border::Above | Border::Below) => {
            let step = if kind == LayoutKind::ThreeColumn {
                2
            } else {
                1
            };
            Some([split, split + step])
        }
        _ => None,
    }
}

/// The windows that `after` puts somewhere else than `before` has them.
pub(super) fn changed(before: &[Rect], after: &[Rect]) -> Vec<usize> {
    let count = before.len().max(after.len());
    (0..count)
        .filter(|&w| before.get(w) != after.get(w))
        .collect()
}

/// The edges of a window held by its `side` edge alone.
pub(super) fn held(side: Side) -> Edges {
    Edges {
        left: side == Side::Left,
        top: side == Side::Top,
        right: side == Side::Right,
        bottom: side == Side::Bottom,
    }
}

/// Where the `side` edge of `rect` is.
pub(super) fn edge_of(rect: Rect, side: Side) -> i32 {
    match side {
        Side::Left => rect.x,
        Side::Top => rect.y,
        Side::Right => rect.x + rect.width,
        Side::Bottom => rect.y + rect.height,
    }
}

/// `rect` with its `side` edge `by` pixels further right or down.
pub(super) fn dragged(rect: Rect, side: Side, by: i32) -> Rect {
    let mut rect = rect;
    match side {
        Side::Left => (rect.x, rect.width) = (rect.x + by, rect.width - by),
        Side::Top => (rect.y, rect.height) = (rect.y + by, rect.height - by),
        Side::Right => rect.width += by,
        Side::Bottom => rect.height += by,
    }
    rect
}
