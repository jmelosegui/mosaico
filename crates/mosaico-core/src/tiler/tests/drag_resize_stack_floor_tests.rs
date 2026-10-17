//! An edge down a stack never moves against the drag, and stops where a
//! window beside it would have less than a tenth of an equal share of the
//! stack, however the edges were dragged before; and what a window that
//! opens after a drag gets, or a monitor that comes back, does not hang on
//! how the stack rounds.

use crate::tiler::tests::{Harness, displays};
use crate::tiler::{BarReservation, Input};
use crate::{Action, LayoutKind, Rect};

use super::super::support::{held, resize_to, resizing};
use super::laid_out;

/// The user drags the bottom edge of window `id` of `h` to `to`, or its
/// top edge with `top`. Returns where the edge was, where it was dropped,
/// and where it is once laid out again.
fn drag_to(h: &mut Harness, id: u64, top: bool, to: i32) -> [i32; 3] {
    let rect = h.rect_of(id);
    let bottom = rect.y + rect.height;
    let edge = |r: Rect| if top { r.y } else { r.y + r.height };
    let dropped = if top {
        Rect::new(rect.x, to, rect.width, bottom - to)
    } else {
        Rect::new(rect.x, rect.y, rect.width, to - rect.y)
    };
    resize_to(h, id, held(false, top, false, !top), dropped);
    [edge(rect), edge(dropped), edge(h.rect_of(id))]
}

/// Whether an edge that was at `was` and was dropped at `dropped` is now
/// between the two: it went with the drag, or stayed.
fn with_the_drag([was, dropped, now]: [i32; 3]) -> bool {
    (was.min(dropped)..=was.max(dropped)).contains(&now)
}

#[test]
fn an_edge_next_to_a_window_at_the_floor_does_not_move_against_the_drag() {
    // Arrange -- windows 2 to 5 stacked, 260 pixels each, the bottom edge
    // of window 3 dragged to its top, which leaves it 26 pixels.
    let mut h = laid_out(LayoutKind::VerticalStack, 5);
    let first = drag_to(&mut h, 3, false, 277);

    // Act -- its top edge dragged down, and the top edge of window 4 a
    // little up, both of which would leave window 3 less.
    let second = drag_to(&mut h, 3, true, 290);
    let third = drag_to(&mut h, 4, true, 300);

    // Assert -- window 3 stays at the floor, and neither edge moves.
    assert_eq!(h.rect_of(3).height, 26);
    assert!([first, second, third].into_iter().all(with_the_drag));
    assert_eq!(
        [second, third].map(|edge| (edge[0], edge[2])),
        [(276, 276), (310, 310)]
    );
}

#[test]
fn edges_dragged_back_and_forth_never_take_a_window_under_the_floor() {
    // Arrange
    let mut h = laid_out(LayoutKind::VerticalStack, 5);

    // Act -- the edge between windows 2 and 3 down, 4 and 5 up, 3 and 4
    // down, 2 and 3 down, and 3 and 4 down, each as far as it goes.
    let drags = [(2, true), (4, false), (3, true), (2, true), (3, true)];
    let mut dragged = Vec::new();
    for (id, down) in drags {
        let top = h.rect_of(id).y;
        let edge = drag_to(&mut h, id, false, if down { 1080 } else { top + 1 });
        let floor = (2..=5).all(|id| h.rect_of(id).height >= 26);
        dragged.push((edge, floor));
    }

    // Assert -- every edge went with its drag, and every window kept the
    // 26 pixels of a tenth of an equal share.
    assert!(
        dragged
            .iter()
            .all(|&(edge, floor)| with_the_drag(edge) && floor)
    );
}

#[test]
fn what_a_window_opening_after_a_drag_gets_does_not_hang_on_how_the_stack_rounds() {
    for gap in [8, 9] {
        // Arrange -- windows 2 to 6 stacked, with a gap that leaves the
        // stack 2 pixels to round, and one that leaves 1, the bottom edge
        // of window 3 dragged 50 down.
        let windows: Vec<(u64, usize)> = (1..=6).map(|id| (id, 0)).collect();
        let mut settings = resizing();
        settings.layout.gap = gap;
        let mut h = Harness::with_settings(settings, 1, &windows);
        while h.tiler.monitors()[0].active().layout_kind() != LayoutKind::VerticalStack {
            h.action(Action::CycleLayout);
        }
        let third = h.rect_of(3);
        drag_to(&mut h, 3, false, third.y + third.height + 50);

        // Act
        h.open(7, 0);

        // Assert -- windows 5, 6 and 7 get about a sixth of the stack each,
        // as windows 5 and 6 had about a fifth.
        let sixth = (1080 - 7 * gap) / 6;
        let heights = [5, 6, 7].map(|id| h.rect_of(id).height);
        assert!(
            heights.iter().all(|h| (h - sixth).abs() <= 2),
            "gap {gap}: {heights:?}"
        );
    }
}

#[test]
fn a_whole_monitor_back_after_a_drag_and_an_open_is_back_to_the_pixel() {
    // Arrange -- windows 2 to 7 in the stack of the second monitor, gap 5,
    // 210 pixels each, the bottom edge of window 4 dragged to the bottom,
    // and window 8 opened, which puts an edge on half a pixel.
    let windows: Vec<(u64, usize)> = [(1, 0)]
        .into_iter()
        .chain((2..=7).map(|id| (id, 1)))
        .collect();
    let mut settings = resizing();
    settings.layout.gap = 5;
    let mut h = Harness::with_settings(settings, 2, &windows);
    let ws = h.tiler.monitors[1].active_ws_mut();
    ws.set_layout_kind(LayoutKind::VerticalStack);
    h.step(Input::Bar(BarReservation::default()));
    drag_to(&mut h, 4, false, 1080);
    h.open(8, 1);
    let before: Vec<Rect> = (2..=8).map(|id| h.rect_of(id)).collect();

    // Act -- it disconnects, and is back.
    for count in [1, 2] {
        let (monitors, bar) = (displays(count), BarReservation::default());
        h.step(Input::Displays { monitors, bar });
    }

    // Assert
    let after: Vec<Rect> = (2..=8).map(|id| h.rect_of(id)).collect();
    assert_eq!(after, before);
}
