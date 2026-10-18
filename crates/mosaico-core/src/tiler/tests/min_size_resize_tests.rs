//! Resizing by the edges stops where a window on either side of the edge
//! would go below its minimum size, live and at the drop.

use super::support::{LEFT, RIGHT, grab_to, held, resize_to, resizing};
use crate::tiler::tests::Harness;
use crate::{Action, LayoutKind, MinSize, Rect};

/// Windows `1..=count` tiled in `kind`, each with the minimum given for it.
fn laid_out(kind: LayoutKind, count: u64, mins: &[(u64, MinSize)]) -> Harness {
    let windows: Vec<(u64, usize)> = (1..=count).map(|id| (id, 0)).collect();
    let mut h = Harness::with_settings(resizing(), 1, &windows);
    for &(id, min) in mins {
        h.window(id).min_size = Some(min);
    }
    h.action(Action::Retile);
    while h.tiler.monitors()[0].active().layout_kind() != kind {
        h.action(Action::CycleLayout);
    }
    h
}

fn wider(rect: Rect, by: i32) -> Rect {
    Rect {
        width: rect.width + by,
        ..rect
    }
}

#[test]
fn the_edge_stops_where_the_window_beyond_keeps_its_minimum_width() {
    // Arrange
    let mut h = laid_out(LayoutKind::Bsp, 2, &[(2, MinSize::new(900, 0))]);
    let left = h.rect_of(1);

    // Act -- 300 pixels to the right, which would leave window 2 648.
    resize_to(&mut h, 1, RIGHT, wider(left, 300));

    // Assert
    assert_eq!(h.rect_of(2), Rect::new(1012, 8, 900, 1064));
    assert_eq!(h.rect_of(1), Rect::new(8, 8, 996, 1064));
}

#[test]
fn the_edge_stops_where_the_window_that_moves_it_keeps_its_minimum_width() {
    // Arrange
    let mut h = laid_out(LayoutKind::Bsp, 2, &[(1, MinSize::new(900, 0))]);
    let right = h.rect_of(2);

    // Act -- the left edge of window 2, 300 pixels to the left.
    let grown = Rect::new(right.x - 300, right.y, right.width + 300, right.height);
    resize_to(&mut h, 2, LEFT, grown);

    // Assert
    assert_eq!(h.rect_of(1), Rect::new(8, 8, 900, 1064));
    assert_eq!(h.rect_of(2), Rect::new(916, 8, 996, 1064));
}

#[test]
fn the_tile_beyond_a_held_edge_follows_it_only_down_to_its_minimum() {
    // Arrange
    let mut h = laid_out(LayoutKind::Bsp, 2, &[(2, MinSize::new(900, 0))]);
    let left = h.rect_of(1);

    // Act -- held, not dropped.
    grab_to(&mut h, 1, RIGHT, wider(left, 300));

    // Assert
    assert_eq!(
        h.placements()[0],
        (2, Rect::new(1012, 8, 900, 1064), crate::tiler::Timing::Now)
    );
}

#[test]
fn a_resize_that_leaves_every_minimum_alone_moves_the_split_as_before() {
    // Arrange
    let mut h = laid_out(LayoutKind::Bsp, 2, &[(2, MinSize::new(400, 0))]);
    let left = h.rect_of(1);

    // Act
    resize_to(&mut h, 1, RIGHT, wider(left, 300));

    // Assert
    assert_eq!(h.rect_of(2), Rect::new(1264, 8, 648, 1064));
}

#[test]
fn a_stack_edge_stops_where_the_window_below_keeps_its_minimum_height() {
    // Arrange -- windows 2, 3 and 4 stacked right of the master.
    let mut h = laid_out(LayoutKind::VerticalStack, 4, &[(4, MinSize::new(0, 400))]);
    let third = h.rect_of(3);

    // Act -- the bottom edge of window 3, 300 pixels down.
    let taller = Rect {
        height: third.height + 300,
        ..third
    };
    resize_to(&mut h, 3, held(false, false, false, true), taller);

    // Assert -- window 4 keeps 400 pixels, down to the work area's end.
    let last = h.rect_of(4);
    assert_eq!(last.height, 400);
    assert_eq!(last.y + last.height, 1072);
}
