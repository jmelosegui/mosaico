//! A window taken in while it is on screen is moved before anything
//! else on its monitor (#28).
//!
//! Such a window shows where the system put it until it is moved. Its
//! first move does not wait for its application, so nothing else waits
//! for it either. Borders are left out of the expected orders: how often
//! a step draws them is a separate question.

use super::adopt_order_support::{first_move_or_border, moves, posted_move};
use super::*;

/// Asserts that a step moved `window` first, without waiting, to the
/// slot it ended in, and still laid it out in the layout pass.
fn assert_moved_first(h: &Harness, effects: &[Effect], window: u64) {
    let first = first_move_or_border(effects);
    assert_eq!(first, Some(format!("post move {window}")), "{effects:?}");
    assert_eq!(posted_move(effects, window), Some(h.rect_of(window)));
    let laid_out = moves(effects)
        .iter()
        .filter(|m| **m == format!("place {window}"))
        .count();
    assert_eq!(laid_out, 1, "the layout pass places it as before");
}

#[test]
fn a_window_created_visible_is_moved_before_the_window_next_to_it() {
    // Arrange -- #28: the window already there was moved and redrawn
    // first, while the new one stayed where the system put it.
    let mut h = Harness::new(1, &[(1, 0)]);

    // Act
    let effects = h.open(99, 0).to_vec();

    // Assert
    assert_moved_first(&h, &effects, 99);
    assert_eq!(h.active(0), vec![1, 99]);
    assert_eq!(h.tiler.focused_window(), Some(wid(99)));
}

#[test]
fn a_window_created_visible_is_moved_before_three_windows_already_there() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0), (2, 0), (3, 0)]);

    // Act
    let effects = h.open(99, 0).to_vec();

    // Assert
    assert_moved_first(&h, &effects, 99);
    assert_eq!(h.active(0), vec![1, 2, 3, 99]);
}

#[test]
fn a_window_restored_from_minimized_is_moved_before_the_windows_there() {
    // Arrange -- it comes back to the end of the layout, in another slot
    let mut h = Harness::new(1, &[(1, 0), (2, 0), (3, 0)]);
    h.window(2).minimized = true;
    h.event(WindowEvent::Minimized { window: wid(2) });
    h.window(2).minimized = false;

    // Act
    let effects = h.event(WindowEvent::Restored { window: wid(2) }).to_vec();

    // Assert
    assert_moved_first(&h, &effects, 2);
    assert_eq!(h.active(0), vec![1, 3, 2]);
}

#[test]
fn a_window_taken_in_when_its_title_is_set_is_moved_before_the_window_there() {
    // Arrange -- the platform rules the window out until it has a title
    let mut h = Harness::new(1, &[(1, 0)]);
    let mut untitled = FakeWindow::app(0);
    untitled.facts = None;
    h.desktop.windows.insert(wid(99), untitled);
    h.event(WindowEvent::Created { window: wid(99) });
    h.window(99).facts = FakeWindow::app(0).facts;

    // Act
    let effects = h
        .event(WindowEvent::TitleChanged { window: wid(99) })
        .to_vec();

    // Assert
    assert_moved_first(&h, &effects, 99);
    assert_eq!(h.active(0), vec![1, 99]);
}

#[test]
fn a_window_taken_in_when_it_gets_focus_is_moved_before_the_window_there() {
    // Arrange -- a window whose creation was never reported
    let mut h = Harness::new(1, &[(1, 0)]);
    h.desktop.windows.insert(wid(99), FakeWindow::app(0));

    // Act
    let effects = h.event(WindowEvent::Focused { window: wid(99) }).to_vec();

    // Assert
    assert_moved_first(&h, &effects, 99);
    assert_eq!(h.active(0), vec![1, 99]);
}

#[test]
fn a_window_taken_in_where_the_user_dropped_it_is_moved_before_the_window_there() {
    // Arrange -- a window the tiler never took in, dragged onto monitor 0
    let mut h = Harness::new(1, &[(1, 0)]);
    h.desktop.windows.insert(wid(99), FakeWindow::app(0));

    // Act
    let effects = h.event(WindowEvent::moved(wid(99))).to_vec();

    // Assert
    assert_moved_first(&h, &effects, 99);
    assert_eq!(h.active(0), vec![1, 99]);
}
