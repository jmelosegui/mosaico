//! The tiles of another monitor in the vertical stack or the three
//! columns while a window is dragged over them: they make room down the
//! stacks as the drop lays them out, heights included, so the drop leaves
//! them where they are.

use crate::LayoutKind;
use crate::Rect;
use crate::tiler::drag::support::{center, drop_at, grab, report_at, swapping};
use crate::tiler::tests::Harness;
use crate::tiler::{BarReservation, Input};

/// Where windows are placed, as `(window, rect)`.
type Placed = Vec<(u64, Rect)>;

/// Windows 1 and 2 on the first monitor, and `ids` on the second in
/// `kind`, with windows dropped on tiles moving there.
fn onto(kind: LayoutKind, ids: &[u64]) -> Harness {
    let mut windows = vec![(1, 0), (2, 0)];
    windows.extend(ids.iter().map(|&id| (id, 1)));
    let mut h = Harness::with_settings(swapping(), 2, &windows);
    h.tiler.monitors[1].active_ws_mut().set_layout_kind(kind);
    h.step(Input::Bar(BarReservation::default()));
    h
}

/// The placements of the last step of `h`, as `(window, rect)`.
fn placed(h: &Harness) -> Placed {
    h.placements().into_iter().map(|(w, r, _)| (w, r)).collect()
}

/// Window 1 of `h` dragged over window `over` on the second monitor, and
/// dropped there. Returns where the report placed each window, and what
/// the drop placed.
fn made_room_and_dropped(h: &mut Harness, over: u64) -> (Placed, Placed) {
    let at = center(h.rect_of(over));
    grab(h, 1);
    report_at(h, 1, at);
    let live = placed(h);
    drop_at(h, 1, at);
    (live, placed(h))
}

#[test]
fn a_vertical_stack_there_makes_room_as_the_drop_lays_it_out() {
    // Arrange -- windows 4 and 5 stacked right of window 3, window 4 with
    // a quarter of the height.
    let mut h = onto(LayoutKind::VerticalStack, &[3, 4, 5]);
    h.tiler.monitors[1]
        .active_ws_mut()
        .set_heights(1, &[0.25, 0.75]);
    h.step(Input::Bar(BarReservation::default()));

    // Act -- over window 5.
    let (live, dropped) = made_room_and_dropped(&mut h, 5);

    // Assert -- windows 4 and 5 make room in the stack as the drop has
    // them, which leaves them there, and window 3 stays.
    let at_drop: Placed = [4, 5].map(|id| (id, h.rect_of(id))).to_vec();
    assert_eq!(live, at_drop);
    assert!(at_drop.iter().all(|placed| dropped.contains(placed)));
    assert_eq!(h.active(1), vec![3, 4, 1, 5]);
}

#[test]
fn three_columns_there_make_room_as_the_drop_lays_them_out() {
    // Arrange -- window 3 in the middle, 4 and 6 on the left, 5 on the
    // right.
    let mut h = onto(LayoutKind::ThreeColumn, &[3, 4, 5, 6]);

    // Act -- over window 6, the last on the left.
    let (live, dropped) = made_room_and_dropped(&mut h, 6);

    // Assert -- window 1 goes on the left, before window 6, which moves
    // under window 5 on the right, as the drop has them, and window 4
    // keeps the top of the left.
    assert_eq!(h.active(1), vec![3, 4, 5, 1, 6]);
    let moved: Vec<u64> = live.iter().map(|&(id, _)| id).collect();
    assert_eq!(moved, vec![5, 6]);
    assert!(live.iter().all(|&(id, rect)| h.rect_of(id) == rect));
    assert!(live.iter().all(|placed| dropped.contains(placed)));
}
