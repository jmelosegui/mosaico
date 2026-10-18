//! What follows a split the user moved with an edge: a window dropped on
//! another takes its slot at its size, cycling layouts comes back to it,
//! and a retile puts it back at the configured ratio.

use crate::tiler::Settings;
use crate::tiler::tests::Harness;
use crate::{Action, Rect};

use super::super::support::{center, drag_to};
use super::support::{RIGHT, resize_to, resizing, unmoved};

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

#[test]
fn a_retile_after_a_resize_puts_the_split_back_at_the_configured_ratio() {
    // Arrange -- where the two are before the resize.
    let mut h = Harness::with_settings(resizing(), 1, &[(1, 0), (2, 0)]);
    let slots = (h.rect_of(1), h.rect_of(2));
    let wider = Rect {
        width: slots.0.width + 300,
        ..slots.0
    };
    resize_to(&mut h, 1, RIGHT, wider);

    // Act -- Alt+Shift+R.
    h.action(Action::Retile);

    // Assert -- both back in their slots at the configured ratio, and the
    // workspace keeps no ratio of its own.
    assert_eq!((h.rect_of(1), h.rect_of(2)), slots);
    assert!(unmoved(&h));
}
