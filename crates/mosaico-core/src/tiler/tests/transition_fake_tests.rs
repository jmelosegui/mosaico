//! What the fake platform does for transitions, which the transition
//! tests rely on: what `FrameControl::place_frames` promises, and frames
//! that land late, are absorbed, or go to a window that is not responding.

use std::time::Duration;

use crate::frames::PENDING_LIMIT;
use crate::id::wid;
use crate::{Rect, WindowControl};

use super::*;

#[path = "transition_fake_order_tests.rs"]
mod order_tests;

/// Window 1 at `x`, in a frame that is the first of its motion when
/// `first`.
fn frame(x: i32, first: bool) -> Frame {
    Frame {
        window: wid(1),
        rect: Rect::new(x, 100, 640, 480),
        first,
    }
}

/// Posts `frames` and returns what the platform found.
fn post(platform: &FakePlatform, frames: &[Frame]) -> Vec<FrameOutcome> {
    let mut outcomes = Vec::new();
    platform
        .place_frames(frames, &mut outcomes)
        .expect("frames are supported");
    outcomes
}

/// Posts window 1's frames one call at a time, `every` apart on the
/// platform's clock, and returns what the platform found of each.
fn post_each(platform: &FakePlatform, frames: &[Frame], every: Duration) -> Vec<FrameOutcome> {
    let mut outcomes = Vec::new();
    for frame in frames {
        outcomes.extend(post(platform, &[*frame]));
        platform.advance(every);
    }
    outcomes
}

#[test]
fn a_frame_lands_at_once_and_the_next_one_finds_it_handled() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let start = platform.window(1).rect;

    // Act
    let frames = [frame(120, true), frame(140, false)];
    let outcomes = post_each(&platform, &frames, Duration::ZERO);

    // Assert
    let reached = frame(120, false).rect;
    let handled = [start, reached].map(|reached| FrameOutcome::Handled { reached });
    assert_eq!(outcomes, handled.to_vec());
    assert_eq!(platform.window(1).rect, frame(140, false).rect);
    assert_eq!(
        platform.calls(),
        vec![
            Call::PlaceFrames(vec![frame(120, true)]),
            Call::PlaceFrames(vec![frame(140, false)]),
        ]
    );
}

#[test]
fn a_slow_window_gets_one_frame_per_limit() {
    // Arrange -- its application takes a hundred calls to get to a frame.
    let platform = FakePlatform::new(1, &[(1, 0)]);
    platform.edit(1, |w| w.frame_delay = 100);

    // Act -- a frame every 50 ms for two limits and a bit.
    let frames: Vec<Frame> = (0..12).map(|i| frame(120 + i, i == 0)).collect();
    let outcomes = post_each(&platform, &frames, Duration::from_millis(50));

    // Assert -- at 0, 250 and 500 ms.
    let posted: Vec<usize> = (0..outcomes.len())
        .filter(|&i| outcomes[i] != FrameOutcome::Pending)
        .collect();
    assert_eq!(posted, vec![0, 5, 10]);
    assert_eq!(platform.window(1).queued.len(), 3);
}

#[test]
fn a_window_that_takes_its_frames_without_moving_gets_one_per_limit() {
    // Arrange -- it puts its size back, or rounds it.
    let platform = FakePlatform::new(1, &[(1, 0)]);
    platform.edit(1, |w| w.absorbs_frames = true);
    let start = platform.window(1).rect;
    post(&platform, &[frame(120, true)]);

    // Act
    let at_once = post(&platform, &[frame(140, false)]);
    platform.advance(PENDING_LIMIT);
    let later = post(&platform, &[frame(160, false)]);

    // Assert -- never held back for longer, and never moved.
    assert_eq!(at_once, vec![FrameOutcome::Pending]);
    assert_eq!(later, vec![FrameOutcome::Handled { reached: start }]);
    assert_eq!(platform.window(1).rect, start);
}

#[test]
fn a_window_whose_application_is_not_responding_stays_pending() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0)]);
    platform.edit(1, |w| w.hung = true);
    let start = platform.window(1).rect;

    // Act
    let first = post(&platform, &[frame(140, true)]);
    platform.advance(Duration::from_secs(5));
    let later = post(&platform, &[frame(160, false)]);

    // Assert
    assert_eq!([first, later].concat(), vec![FrameOutcome::Pending; 2]);
    assert_eq!(platform.window(1).rect, start);
}

#[test]
fn windows_that_are_minimized_maximized_or_gone_are_left_alone() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0), (3, 0)]);
    platform.edit(1, |w| w.minimized = true);
    platform.edit(2, |w| w.maximized = true);
    platform.close_window(3);
    let before = [platform.window(1).rect, platform.window(2).rect];
    let rect = Rect::new(0, 0, 800, 600);
    let frames = [1, 2, 3].map(|id| Frame {
        window: wid(id),
        rect,
        first: true,
    });

    // Act
    let outcomes = post(&platform, &frames);

    // Assert
    use FrameOutcome::{Gone, Maximized, Minimized};
    assert_eq!(outcomes, vec![Minimized, Maximized, Gone]);
    assert_eq!([platform.window(1).rect, platform.window(2).rect], before);
    assert!(platform.window(1).minimized);
    assert!(platform.window(2).maximized);
}
