//! How often a hidden window of a thread of the test's own gets frames:
//! one per limit at least while it does not move, whether its thread is
//! slow or it takes its frames without moving, and at once after it moves.

use std::thread;
use std::time::{Duration, Instant};

use mosaico_core::WindowControl;
use mosaico_core::frames::PENDING_LIMIT;

use crate::window::test_threads::{ThreadWindow, within_a_second};
use crate::window::test_windows::Takes;

use super::*;

/// A post, the times around it, and what it found.
type Posted = (Instant, Instant, FrameOutcome);

/// Posts `window` a frame every 10 ms for `run`, each a pixel further
/// right and wider than the last, and returns what was found of each,
/// with the times around it, which bracket the table's own.
fn post_for(control: &Win32WindowControl, window: WindowId, run: Duration) -> Vec<Posted> {
    let started = Instant::now();
    let mut found = Vec::new();
    for step in 0.. {
        if started.elapsed() >= run {
            break;
        }
        let rect = Rect::new(40 + step, 30, 250 + step, 90);
        let before = Instant::now();
        let outcome = post(control, frame(window, rect, step == 0));
        found.push((before, Instant::now(), outcome));
        thread::sleep(Duration::from_millis(10));
    }
    found
}

/// Asserts that no frame was held back past the limit after the last one
/// posted, and that more than one was posted.
fn assert_never_held_back_past_the_limit(found: &[Posted]) {
    let mut last_post = None;
    for &(before, after, outcome) in found {
        if outcome == FrameOutcome::Pending {
            let since = before.saturating_duration_since(last_post.expect("a frame went first"));
            assert!(since < PENDING_LIMIT, "held back for {since:?}");
        } else {
            last_post = Some(after);
        }
    }
    let posted = found.iter().filter(|(_, _, o)| *o != FrameOutcome::Pending);
    assert!(posted.count() > 1, "{found:?}");
}

#[test]
fn a_slow_window_gets_one_frame_per_limit() {
    // Arrange -- its thread never gets to its messages.
    let Some(owned) = ThreadWindow::new() else {
        return;
    };
    let window = owned.id();

    // Act
    let found = within_a_second(move || {
        let control = Win32WindowControl::default();
        post_for(
            &control,
            window,
            PENDING_LIMIT * 2 + Duration::from_millis(50),
        )
    });
    drop(owned);

    // Assert -- and the frames posted were a limit apart at least.
    let found = found.expect("place_frames returned at once");
    assert_never_held_back_past_the_limit(&found);
    let posts: Vec<&Posted> = found
        .iter()
        .filter(|p| p.2 != FrameOutcome::Pending)
        .collect();
    for pair in posts.windows(2) {
        let apart = pair[1].1.duration_since(pair[0].0);
        assert!(apart >= PENDING_LIMIT, "posted {apart:?} apart");
    }
}

/// Posts a window that takes its moves as `takes` frames for a while,
/// then places and repaints it, and returns what was found of the frames
/// and whether the placement and the repaint returned at once.
fn frames_then_place(takes: Takes) -> Option<(Vec<Posted>, bool)> {
    let owned = ThreadWindow::taking(takes)?;
    owned.pump();
    let window = owned.id();
    let found = within_a_second(move || {
        let control = Win32WindowControl::default();
        let found = post_for(&control, window, PENDING_LIMIT * 2);
        let target = Rect::new(400, 30, 250, 90);
        let finished = control.place(window, target).is_ok() && control.repaint(window).is_ok();
        (found, finished)
    });
    drop(owned);
    Some(found.expect("nothing waited for the window"))
}

#[test]
fn a_window_that_puts_its_size_back_never_holds_frames_back_past_the_limit() {
    // Arrange / Act -- it never shows a move.
    let Some((found, finished)) = frames_then_place(Takes::PutsBack) else {
        return;
    };

    // Assert
    assert_never_held_back_past_the_limit(&found);
    let held = found
        .iter()
        .any(|(_, _, outcome)| *outcome == FrameOutcome::Pending);
    assert!(held, "it was seen to move");
    assert!(finished, "placing or repainting it failed");
}

#[test]
fn a_window_that_rounds_its_size_never_holds_frames_back_past_the_limit() {
    // Arrange / Act -- it never reaches the size it is asked for.
    let Some((found, finished)) = frames_then_place(Takes::RoundsSize) else {
        return;
    };

    // Assert
    assert_never_held_back_past_the_limit(&found);
    assert!(finished, "placing or repainting it failed");
}
