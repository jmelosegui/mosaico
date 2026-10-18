//! Resizes of a window whose grab the platform cannot tell, as a window
//! that runs its own move and resize loops has it: the edges held are read
//! from how the drag changed the window.

use crate::event::{DragStart, Edges};
use crate::id::wid;
use crate::tiler::tests::Harness;
use crate::{LayoutKind, Rect, WindowEvent};

use super::support::{RIGHT, holding, resizing};

/// The user grabs window `id` of `h` by `edges`, which the platform cannot
/// tell, resizes it to `rect`, which the window reports, and lets go there.
fn resize_unknown(h: &mut Harness, id: u64, edges: Edges, rect: Rect) {
    let (window, start) = (wid(id), h.rect_of(id));
    h.event(WindowEvent::Focused { window });
    let start = DragStart {
        grab: None,
        rect: Some(start),
        maximized: Some(false),
        pointer: Some(holding(start, edges)),
    };
    h.event(WindowEvent::MoveStarted { window, start });
    h.window(id).rect = rect;
    h.event(WindowEvent::LocationChanged { window });
    let pointer = Some(holding(rect, edges));
    h.event(WindowEvent::Moved { window, pointer });
}

/// The ratio of split `k` of the BSP chain on the second monitor of `h`.
fn split(h: &Harness, k: usize) -> Option<f64> {
    h.workspace(1, 0).split_ratios().ratio(LayoutKind::Bsp, k)
}

#[test]
fn a_window_that_reports_no_grab_resized_by_its_right_edge_moves_the_split() {
    // Arrange -- windows 1 and 2 side by side on the second monitor, as
    // the report has them on monitor 1.
    let mut h = Harness::with_settings(resizing(), 2, &[(1, 1), (2, 1)]);
    let (left, right) = (h.rect_of(1), h.rect_of(2));
    let gap = right.x - (left.x + left.width);
    let wider = Rect {
        width: left.width + 300,
        ..left
    };

    // Act -- its right edge 300 pixels right, with no grab reported.
    resize_unknown(&mut h, 1, RIGHT, wider);

    // Assert -- the split follows the edge: window 1 stays as it was
    // dropped, and window 2 starts a gap after it.
    assert_eq!(h.rect_of(1), wider);
    assert_eq!(h.rect_of(2).x, wider.x + wider.width + gap);
    assert!(split(&h, 0).is_some());
}
