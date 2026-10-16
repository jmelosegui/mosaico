//! `place_frames` on hidden windows of threads of the test's own, which
//! handle their messages when the test tells them to, or never.
//!
//! A frame posted right after another to a window that has not moved is
//! held back only for `PENDING_LIMIT`, so the tests measure how long they
//! took and expect it held back only when that was less: a test thread
//! stalled that long between two posts sees the second one go.

use mosaico_core::frames::PENDING_LIMIT;

use crate::window::test_threads::{ThreadWindow, moved_from, within_a_second};

use super::*;

const C: Rect = Rect {
    x: 80,
    y: 40,
    width: 100,
    height: 80,
};

#[test]
fn a_window_that_never_handles_its_frame_gets_no_other_within_the_limit() {
    // Arrange
    let Some(owned) = ThreadWindow::new() else {
        return;
    };
    let window = owned.id();
    let start = rect_of(window).expect("the window exists");

    // Act
    let found = within_a_second(move || {
        let control = Win32WindowControl::default();
        timed(|| [A, B, C].map(|rect| post(&control, frame(window, rect, rect == A))))
    });
    let end = rect_of(window);
    drop(owned);

    // Assert -- and it never moved.
    let (found, took) = found.expect("place_frames returned at once");
    assert_eq!(found[0], FrameOutcome::Handled { reached: start });
    if took < PENDING_LIMIT {
        assert_eq!(found[1..], [FrameOutcome::Pending; 2]);
    }
    assert_eq!(end, Some(start));
}

#[test]
fn a_frame_that_lands_late_lets_the_next_one_go_once_the_window_has_moved() {
    // Arrange -- the window's thread gets to its messages when told.
    let Some(owned) = ThreadWindow::new() else {
        return;
    };
    let window = owned.id();
    let control = Win32WindowControl::default();
    let start = rect_of(window);

    // Act -- B is posted once A has landed.
    let (early, took) = timed(|| {
        post(&control, frame(window, A, true));
        post(&control, frame(window, B, false))
    });
    owned.pump();
    let a_landed = moved_from(window, start);
    let at_a = rect_of(window);
    let landed = post(&control, frame(window, B, false));
    let b_landed = moved_from(window, at_a);
    let last = post(&control, frame(window, B, false));
    drop(owned);

    // Assert -- the window ends where B put it.
    if took < PENDING_LIMIT {
        assert_eq!(early, FrameOutcome::Pending);
    }
    assert!(a_landed && b_landed, "a frame never landed");
    assert_eq!(landed, FrameOutcome::Handled { reached: A });
    assert_eq!(last, FrameOutcome::Handled { reached: B });
}

#[test]
fn a_new_motion_is_held_back_by_a_frame_of_the_last_one_on_its_way() {
    // Arrange
    let Some(owned) = ThreadWindow::new() else {
        return;
    };
    let window = owned.id();
    let start = rect_of(window).expect("the window exists");

    // Act -- the new motion comes while a frame of the old one is still
    // on its way.
    let found = within_a_second(move || {
        let control = Win32WindowControl::default();
        let frames = [(A, true), (B, false), (C, true)];
        timed(|| frames.map(|(rect, first)| post(&control, frame(window, rect, first))))
    });
    drop(owned);

    // Assert
    let (found, took) = found.expect("place_frames returned at once");
    assert_eq!(found[0], FrameOutcome::Handled { reached: start });
    if took < PENDING_LIMIT {
        assert_eq!(found[1..], [FrameOutcome::Pending; 2]);
    }
}
