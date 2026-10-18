//! Resizing tiled windows by their edges in the vertical stack and in
//! three columns, with `resize_splits` on: the master's edges move the
//! master split. The edges down a stack or a side column are in
//! `drag_resize_stack_tests.rs` and `drag_resize_stack_cases_tests.rs`.

use crate::tiler::tests::Harness;
use crate::{Action, LayoutKind, Rect};

use super::support::{held, resize_to, resizing};

/// Windows `1..=count` tiled on one monitor with `kind`, with splits moved
/// by the edges of the windows the user resizes.
fn laid_out(kind: LayoutKind, count: u64) -> Harness {
    let windows: Vec<(u64, usize)> = (1..=count).map(|id| (id, 0)).collect();
    let mut h = Harness::with_settings(resizing(), 1, &windows);
    while h.tiler.monitors()[0].active().layout_kind() != kind {
        h.action(Action::CycleLayout);
    }
    h
}

#[test]
fn the_edge_of_the_master_of_the_stack_moves_the_whole_stack() {
    // Arrange
    let mut h = laid_out(LayoutKind::VerticalStack, 3);
    let master = h.rect_of(1);

    // Act -- its right edge 200 pixels right.
    let wider = Rect {
        width: master.width + 200,
        ..master
    };
    resize_to(&mut h, 1, held(false, false, true, false), wider);

    // Assert
    assert_eq!(h.rect_of(1), wider);
    assert_eq!(h.rect_of(2), Rect::new(1164, 8, 748, 528));
    assert_eq!(h.rect_of(3), Rect::new(1164, 544, 748, 528));
}

#[test]
fn either_edge_of_the_master_of_three_columns_resizes_it_from_the_center() {
    // Arrange -- the master between the two columns.
    let mut h = laid_out(LayoutKind::ThreeColumn, 3);
    let master = h.rect_of(1);

    // Act -- its right edge 100 pixels right.
    let wider = Rect {
        width: master.width + 100,
        ..master
    };
    resize_to(&mut h, 1, held(false, false, true, false), wider);

    // Assert -- its left edge goes 100 pixels left, and both columns give
    // up the width.
    assert_eq!(h.rect_of(1), Rect::new(384, 8, 1152, 1064));
    assert_eq!(h.rect_of(2), Rect::new(8, 8, 368, 1064));
    assert_eq!(h.rect_of(3), Rect::new(1544, 8, 368, 1064));
}

#[test]
fn the_inner_edge_of_a_column_moves_both_edges_of_the_master() {
    // Arrange -- window 2 is the left column.
    let mut h = laid_out(LayoutKind::ThreeColumn, 3);
    let column = h.rect_of(2);

    // Act -- its right edge 50 pixels right.
    let wider = Rect {
        width: column.width + 50,
        ..column
    };
    resize_to(&mut h, 2, held(false, false, true, false), wider);

    // Assert
    assert_eq!(h.rect_of(2), wider);
    assert_eq!(h.rect_of(1), Rect::new(534, 8, 852, 1064));
    assert_eq!(h.rect_of(3), Rect::new(1394, 8, 518, 1064));
}

#[test]
fn two_windows_in_three_columns_split_as_in_the_stack() {
    // Arrange
    let mut h = laid_out(LayoutKind::ThreeColumn, 2);
    let right = h.rect_of(2);

    // Act -- the left edge of the second, 200 pixels left.
    let wider = Rect::new(right.x - 200, right.y, right.width + 200, right.height);
    resize_to(&mut h, 2, held(true, false, false, false), wider);

    // Assert
    assert_eq!(h.rect_of(2), wider);
    assert_eq!(h.rect_of(1), Rect::new(8, 8, 748, 1064));
}

#[path = "drag_resize_stack_tests.rs"]
mod stack_tests;

#[path = "drag_resize_stack_cases_tests.rs"]
mod stack_cases_tests;

#[path = "drag_resize_stack_open_tests.rs"]
mod stack_open_tests;

#[path = "drag_resize_stack_floor_tests.rs"]
mod stack_floor_tests;
