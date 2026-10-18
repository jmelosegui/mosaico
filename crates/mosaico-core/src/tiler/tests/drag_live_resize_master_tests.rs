//! The tiles that follow the master's edge while the user drags it, with
//! `resize_splits` on: the whole stack in VerticalStack, and both side
//! columns in ThreeColumn, where the master stays centered.

use crate::tiler::Timing;
use crate::tiler::drag::resize::support::{RIGHT, grab_to, resizing, unmoved};
use crate::tiler::tests::Harness;
use crate::{Action, LayoutKind, Rect};

/// Windows `1..=count` tiled on one monitor in `kind`, with splits moved
/// by the edges of the windows the user resizes.
fn laid_out(kind: LayoutKind, count: u64) -> Harness {
    let windows: Vec<(u64, usize)> = (1..=count).map(|id| (id, 0)).collect();
    let mut h = Harness::with_settings(resizing(), 1, &windows);
    while h.tiler.monitors()[0].active().layout_kind() != kind {
        h.action(Action::CycleLayout);
    }
    h
}

/// `rect` with its left edge moved to `x`, its right edge where it was.
fn from(rect: Rect, x: i32) -> Rect {
    Rect::new(x, rect.y, rect.x + rect.width - x, rect.height)
}

/// `rect` with its right edge moved to `right`, its left edge where it was.
fn to(rect: Rect, right: i32) -> Rect {
    Rect::new(rect.x, rect.y, right - rect.x, rect.height)
}

#[test]
fn the_whole_stack_follows_the_master_s_right_edge() {
    // Arrange -- windows 2, 3 and 4 stacked right of the master.
    let mut h = laid_out(LayoutKind::VerticalStack, 4);
    let master = h.rect_of(1);
    let stack: Vec<Rect> = (2..=4).map(|id| h.rect_of(id)).collect();

    // Act -- the master's right edge, 200 pixels to the right.
    let wider = Rect {
        width: master.width + 200,
        ..master
    };
    grab_to(&mut h, 1, RIGHT, wider);

    // Assert -- each window of the stack starts a gap right of it.
    let x = wider.x + wider.width + 8;
    let placed: Vec<_> = (2..=4)
        .zip(stack)
        .map(|(id, rect)| (id, from(rect, x), Timing::Now))
        .collect();
    assert_eq!(h.placements(), placed);
    assert!(unmoved(&h));
}

#[test]
fn both_side_columns_follow_the_master_s_right_edge_in_three_columns() {
    // Arrange -- the master in the middle, windows 2 and 4 left of it,
    // 3 and 5 right of it.
    let mut h = laid_out(LayoutKind::ThreeColumn, 5);
    let master = h.rect_of(1);
    let sides: Vec<Rect> = (2..=5).map(|id| h.rect_of(id)).collect();

    // Act -- the master's right edge, 100 pixels to the right.
    let wider = Rect {
        width: master.width + 100,
        ..master
    };
    grab_to(&mut h, 1, RIGHT, wider);

    // Assert -- the master stays centered, so its left edge goes 100
    // pixels left too: the left column ends a gap before it, and the
    // right one starts a gap after the edge the user holds.
    let (left, right) = (master.x - 100 - 8, wider.x + wider.width + 8);
    let placed: Vec<_> = (2..=5)
        .zip(sides)
        .map(|(id, rect)| {
            let moved = if id % 2 == 0 {
                to(rect, left)
            } else {
                from(rect, right)
            };
            (id, moved, Timing::Now)
        })
        .collect();
    let mut placements = h.placements();
    placements.sort_by_key(|&(id, ..)| id);
    assert_eq!(placements, placed);
    assert!(unmoved(&h));
}
