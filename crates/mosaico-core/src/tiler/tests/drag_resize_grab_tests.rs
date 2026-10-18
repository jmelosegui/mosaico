//! Which edges a drag held, read from how it changed a window whose grab
//! the platform cannot tell.

use crate::Rect;
use crate::event::Edges;

use super::inferred_edges;

/// Where the window was as the drag started.
fn start() -> Rect {
    Rect::new(100, 100, 400, 300)
}

/// The window at [`start`], its edges moved by `left`, `top`, `right` and
/// `bottom` pixels.
fn moved(left: i32, top: i32, right: i32, bottom: i32) -> Rect {
    let s = start();
    let (width, height) = (s.width + right - left, s.height + bottom - top);
    Rect::new(s.x + left, s.y + top, width, height)
}

/// The edges named, held.
fn held(left: bool, top: bool, right: bool, bottom: bool) -> Option<Edges> {
    Some(Edges {
        left,
        top,
        right,
        bottom,
    })
}

#[test]
fn the_edge_that_moved_is_held() {
    // Arrange
    let cases = [
        (moved(-50, 0, 0, 0), held(true, false, false, false)),
        (moved(0, -50, 0, 0), held(false, true, false, false)),
        (moved(0, 0, 50, 0), held(false, false, true, false)),
        (moved(0, 0, 0, 50), held(false, false, false, true)),
    ];

    for (rect, edges) in cases {
        // Act
        let inferred = inferred_edges(start(), rect);

        // Assert
        assert_eq!(inferred, edges, "{rect:?}");
    }
}

#[test]
fn a_corner_holds_one_edge_of_each_axis() {
    // Arrange
    let (bottom_left, top_right) = (moved(-50, 0, 0, 30), moved(0, -30, 50, 0));

    // Act
    let inferred = [bottom_left, top_right].map(|rect| inferred_edges(start(), rect));

    // Assert
    let corners = [
        held(true, false, false, true),
        held(false, true, true, false),
    ];
    assert_eq!(inferred, corners);
}

#[test]
fn an_edge_that_moved_no_more_than_a_press_wobbles_is_not_held() {
    // Arrange -- 4 pixels is a wobble, 5 a resize.
    let cases = [
        (moved(0, 0, 4, 0), None),
        (moved(0, 0, 5, 0), held(false, false, true, false)),
        (moved(0, 4, 5, 0), held(false, false, true, false)),
    ];

    for (rect, edges) in cases {
        // Act
        let inferred = inferred_edges(start(), rect);

        // Assert
        assert_eq!(inferred, edges, "{rect:?}");
    }
}

#[test]
fn a_drag_that_moved_both_edges_of_an_axis_or_no_edge_holds_none() {
    // Arrange -- a move, a move with a rescale, a resize on one axis with
    // a move on the other, both edges just past a wobble, and nothing.
    let cases = [
        moved(100, 0, 100, 0),
        moved(100, 0, 150, 0),
        moved(0, 30, 50, 30),
        moved(-5, 0, 5, 0),
        moved(-4, 4, 4, -4),
    ];

    for rect in cases {
        // Act
        let inferred = inferred_edges(start(), rect);

        // Assert
        assert_eq!(inferred, None, "{rect:?}");
    }
}
