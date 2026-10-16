//! How long a window may go without being seen to move before it has
//! fallen behind: counted from the first frame it handled, up to and
//! including [`MAX_LAG`].

use std::time::Instant;

use crate::Rect;
use crate::frames::FrameOutcome;
use crate::id::wid;
use crate::motion::Easing;
use crate::motion::test_support::{ms, spec};

use super::{MAX_LAG, Motions, Tick};

/// The left half of the screen, and the right one.
const LEFT: Rect = Rect {
    x: 0,
    y: 0,
    width: 800,
    height: 900,
};
const RIGHT: Rect = Rect { x: 800, ..LEFT };

/// Window 1 sent from the left half to the right one at `t0`, its first
/// frame handled `first` ms in.
fn first_handled(t0: Instant, first: u64) -> Motions {
    let mut motions = Motions::default();
    motions.retarget(wid(1), LEFT, RIGHT, t0, spec(500, Easing::Linear));
    let mut tick = motions.frame(t0 + ms(first));
    let handled = FrameOutcome::Handled { reached: LEFT };
    motions.take(&mut tick, &[handled], t0 + ms(first));
    motions
}

/// A frame `at` ms after `t0` that finds window 1 still to handle the one
/// before.
fn pending(motions: &mut Motions, t0: Instant, at: u64) -> Tick {
    let mut tick = motions.frame(t0 + ms(at));
    motions.take(&mut tick, &[FrameOutcome::Pending], t0 + ms(at));
    tick
}

#[test]
fn the_time_before_the_first_frame_does_not_count_as_lag() {
    // Arrange -- the first frame comes 60 ms after the motion starts.
    let t0 = Instant::now();
    let mut motions = first_handled(t0, 60);

    // Act -- the next finds the window still to handle the first.
    let next = pending(&mut motions, t0, 76);

    // Assert -- 16 ms since the first frame is no lag.
    assert!(next.finished.is_empty());
    assert!(motions.is_moving(wid(1)));
}

#[test]
fn a_window_not_seen_to_move_for_just_the_limit_has_fallen_behind() {
    // Arrange
    let t0 = Instant::now();
    let mut motions = first_handled(t0, 10);
    let limit = u64::try_from(MAX_LAG.as_millis()).expect("a few ms");

    // Act
    let before = pending(&mut motions, t0, 10 + limit - 1);
    let at = pending(&mut motions, t0, 10 + limit);

    // Assert
    assert!(before.finished.is_empty());
    assert_eq!(at.finished, [(wid(1), RIGHT)]);
}
