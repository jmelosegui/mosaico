//! Moving a window to the next monitor puts it on the side it came from,
//! whatever the layout there.

use crate::{Direction, LayoutKind};

use super::*;

const LAYOUTS: [LayoutKind; 3] = [
    LayoutKind::Bsp,
    LayoutKind::VerticalStack,
    LayoutKind::ThreeColumn,
];

/// Two monitors in `layout`, with `left` windows on the first, numbered
/// from 1, and `right` windows on the second, numbered from 101.
fn both_in(layout: LayoutKind, left: u64, right: u64) -> Harness {
    let windows: Vec<(u64, usize)> = (1..=left)
        .map(|id| (id, 0))
        .chain((101..101 + right).map(|id| (id, 1)))
        .collect();
    let mut h = Harness::new(2, &windows);
    for mon in 0..2 {
        h.tiler.monitors[mon]
            .active_ws_mut()
            .set_layout_kind(layout);
    }
    h.action(Action::Retile);
    h
}

/// The left edge of the leftmost window on monitor `mon`, and the right
/// edge of the rightmost one.
fn edges(h: &Harness, mon: usize) -> (i32, i32) {
    let rects: Vec<Rect> = h.active(mon).into_iter().map(|w| h.rect_of(w)).collect();
    let left = rects.iter().map(|r| r.x).min().unwrap_or_default();
    let right = rects
        .iter()
        .map(|r| r.x + r.width)
        .max()
        .unwrap_or_default();
    (left, right)
}

#[test]
fn a_window_moved_right_onto_three_columns_lands_in_the_left_column() {
    // Arrange -- both monitors in three columns, with window 3, in the
    // right column of the first, focused.
    let mut h = both_in(LayoutKind::ThreeColumn, 3, 3);
    h.event(WindowEvent::Focused { window: wid(3) });
    let (left_edge, _) = edges(&h, 1);

    // Act
    h.action(Action::Move(Direction::Right));

    // Assert
    assert!(h.active(1).contains(&3));
    assert_eq!(
        h.rect_of(3).x,
        left_edge,
        "window 3 lands in the left column, not in the master's"
    );
}

#[test]
fn a_window_moved_across_monitors_lands_on_the_side_it_came_from() {
    for layout in LAYOUTS {
        for there in 0..=5 {
            // Moving right, from the first monitor onto the second.
            let mut h = both_in(layout, 2, there);
            h.event(WindowEvent::Focused { window: wid(2) });
            h.action(Action::Move(Direction::Right));
            let (left_edge, _) = edges(&h, 1);
            assert_eq!(
                h.rect_of(2).x,
                left_edge,
                "{layout:?} with {there} windows there: moved right, it enters on the left"
            );

            // Moving left, from the second monitor onto the first.
            let mut h = both_in(layout, there, 2);
            h.event(WindowEvent::Focused { window: wid(101) });
            h.action(Action::Move(Direction::Left));
            let (_, right_edge) = edges(&h, 0);
            let rect = h.rect_of(101);
            assert_eq!(
                rect.x + rect.width,
                right_edge,
                "{layout:?} with {there} windows there: moved left, it enters on the right"
            );
        }
    }
}
