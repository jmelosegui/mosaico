//! What follows a split the user moved with an edge: a window dropped on
//! another takes its slot at its size, and cycling layouts comes back to
//! it.

use crate::tiler::Settings;
use crate::tiler::tests::Harness;
use crate::{Action, Rect};

use super::super::support::{center, drag_to};
use super::support::{RIGHT, resize_to, resizing};

/// Windows 1 and 2 side by side, with `settings`, and window 1 made 300
/// pixels wider by its right edge. Returns where window 1 is then too.
fn widened(settings: Settings) -> (Harness, Rect) {
    let mut h = Harness::with_settings(settings, 1, &[(1, 0), (2, 0)]);
    let left = h.rect_of(1);
    let wider = Rect {
        width: left.width + 300,
        ..left
    };
    resize_to(&mut h, 1, RIGHT, wider);
    (h, wider)
}

#[test]
fn a_window_dropped_on_another_after_a_resize_takes_its_slot_and_its_size() {
    // Arrange -- with drops moving windows too.
    let settings = Settings {
        drag_to_swap: true,
        ..resizing()
    };
    let (mut h, wider) = widened(settings);
    let narrow = h.rect_of(2);

    // Act -- window 2 dragged onto window 1.
    drag_to(&mut h, 2, center(wider));

    // Assert -- the slots keep their sizes.
    assert_eq!(h.active(0), vec![2, 1]);
    assert_eq!((h.rect_of(2), h.rect_of(1)), (wider, narrow));
}

#[test]
fn a_split_moved_by_an_edge_is_still_there_once_the_layouts_come_around() {
    // Arrange
    let (mut h, wider) = widened(resizing());

    // Act -- through the stack and the three columns, back to BSP.
    let others: Vec<Rect> = (0..2)
        .map(|_| {
            h.action(Action::CycleLayout);
            h.rect_of(1)
        })
        .collect();
    h.action(Action::CycleLayout);

    // Assert -- the other layouts kept their own.
    assert!(others.iter().all(|&rect| rect != wider), "{others:?}");
    assert_eq!(h.rect_of(1), wider);
}
