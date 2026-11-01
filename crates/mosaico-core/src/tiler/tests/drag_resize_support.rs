//! What the resize tests share: the user resizing a tiled window by its
//! edges, and whether a split moved.

use crate::event::{DragStart, Edges, Grab};
use crate::id::wid;
use crate::layout::SplitRatios;
use crate::tiler::tests::Harness;
use crate::tiler::{Effect, Settings};
use crate::{Rect, WindowEvent};

/// The right edge of a window, held alone.
pub(in crate::tiler) const RIGHT: Edges = held(false, false, true, false);

/// The left edge of a window, held alone.
pub(in crate::tiler) const LEFT: Edges = held(true, false, false, false);

/// Settings with the edges of a window the user resizes moving the splits
/// they border.
pub(in crate::tiler) fn resizing() -> Settings {
    Settings {
        resize_splits: true,
        ..Settings::with_gap(8)
    }
}

/// The edges of a window held at once.
pub(in crate::tiler) const fn held(left: bool, top: bool, right: bool, bottom: bool) -> Edges {
    Edges {
        left,
        top,
        right,
        bottom,
    }
}

/// The user grabs window `id` by `edges` where it is, which gives it
/// focus, and resizes it to `rect`, which the window reports, without
/// letting go.
pub(in crate::tiler) fn grab_to(h: &mut Harness, id: u64, edges: Edges, rect: Rect) {
    grab_by(h, id, Some(Grab::Edges(edges)), edges, rect);
}

/// As [`grab_to`], with the platform telling `grab` of the drag: the
/// edges, or nothing, as for a window that runs its own resize loop.
pub(in crate::tiler) fn grab_by(
    h: &mut Harness,
    id: u64,
    grab: Option<Grab>,
    edges: Edges,
    rect: Rect,
) {
    let (window, start) = (wid(id), h.rect_of(id));
    h.event(WindowEvent::Focused { window });
    let start = DragStart {
        grab,
        rect: Some(start),
        maximized: Some(false),
        pointer: Some(holding(start, edges)),
    };
    h.event(WindowEvent::MoveStarted { window, start });
    h.window(id).rect = rect;
    h.event(WindowEvent::LocationChanged { window });
}

/// The user resizes window `id` by `edges` to `rect`, as [`grab_to`]
/// does, and lets go of it there. Returns what letting go does.
pub(in crate::tiler) fn resize_to(
    h: &mut Harness,
    id: u64,
    edges: Edges,
    rect: Rect,
) -> Vec<Effect> {
    grab_to(h, id, edges, rect);
    let_go(h, id, edges, rect)
}

/// As [`resize_to`], for a window whose grab the platform cannot tell.
pub(in crate::tiler) fn resize_untold(
    h: &mut Harness,
    id: u64,
    edges: Edges,
    rect: Rect,
) -> Vec<Effect> {
    grab_by(h, id, None, edges, rect);
    let_go(h, id, edges, rect)
}

/// The user lets go of window `id`, held by `edges` at `rect`. Returns
/// what letting go does.
fn let_go(h: &mut Harness, id: u64, edges: Edges, rect: Rect) -> Vec<Effect> {
    let pointer = Some(holding(rect, edges));
    let window = wid(id);
    h.event(WindowEvent::Moved { window, pointer }).to_vec()
}

/// Where the pointer holds a window at `rect` by `edges`: on the edge, or
/// at the corner.
pub(in crate::tiler) fn holding(rect: Rect, edges: Edges) -> (i32, i32) {
    let along = |near: bool, far: bool, start: i32, len: i32| match (near, far) {
        (true, _) => start,
        (_, true) => start + len - 1,
        _ => start + len / 2,
    };
    let x = along(edges.left, edges.right, rect.x, rect.width);
    let y = along(edges.top, edges.bottom, rect.y, rect.height);
    (x, y)
}

/// Whether the splits of the workspace on screen of the first monitor of
/// `h` are where the layout puts them unless the user moves them.
pub(in crate::tiler) fn unmoved(h: &Harness) -> bool {
    let ws = h.tiler.monitors()[0].active();
    let mut fresh = SplitRatios::default();
    (0..ws.len()).for_each(|index| fresh.insert(index, index));
    *ws.split_ratios() == fresh
}
