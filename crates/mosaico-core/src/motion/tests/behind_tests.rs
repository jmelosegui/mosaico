//! What the platform found of the moving windows: windows that fell
//! behind, or left.

use std::time::Instant;

use crate::Rect;
use crate::frames::{Frame, FrameOutcome};
use crate::id::wid;
use crate::motion::Easing;
use crate::motion::test_support::{ms, spec};

use super::{Motions, Tick};

/// The top left quarter of the screen, and the three other ones.
const A: Rect = Rect {
    x: 0,
    y: 0,
    width: 800,
    height: 450,
};
const B: Rect = Rect { x: 800, ..A };
const C: Rect = Rect { y: 450, ..A };
const D: Rect = Rect {
    x: 800,
    y: 450,
    ..A
};

/// A frame `at` ms after `t0`, finding window 2 still to handle its last.
fn pending_2(motions: &mut Motions, t0: Instant, at: u64) -> Tick {
    let mut tick = motions.frame(t0 + ms(at));
    let found = |frame: &Frame| match frame.window == wid(2) {
        true => FrameOutcome::Pending,
        false => FrameOutcome::Handled {
            reached: frame.rect,
        },
    };
    let outcomes: Vec<_> = tick.in_flight.iter().map(found).collect();
    motions.take(&mut tick, &outcomes, t0 + ms(at));
    tick
}

#[test]
fn a_window_that_falls_behind_ends_every_motion_of_its_step_and_no_other() {
    // Arrange -- windows 1 and 2 swap in one step, window 3 moves in the
    // next.
    let t0 = Instant::now();
    let slow = spec(500, Easing::Linear);
    let mut motions = Motions::default();
    motions.begin_step();
    motions.retarget(wid(1), A, B, t0, slow);
    motions.retarget(wid(2), B, A, t0, slow);
    motions.begin_step();
    motions.retarget(wid(3), C, D, t0, slow);

    // Act -- frames at 60 Hz find window 2 still to handle the one before.
    let ticks = [16, 32, 48, 64].map(|at| pending_2(&mut motions, t0, at));

    // Assert -- once it has not been seen to move for the limit, windows 1
    // and 2 are done, and window 3 goes on along its curve.
    assert!(ticks[..3].iter().all(|tick| tick.finished.is_empty()));
    assert_eq!(ticks[3].finished, [(wid(1), B), (wid(2), A)]);
    assert!(motions.is_moving(wid(3)));
}

#[test]
fn a_window_falls_behind_after_the_same_time_at_a_higher_refresh_rate() {
    // Arrange -- windows 1 and 2 swap, framed at 144 Hz.
    let t0 = Instant::now();
    let mut motions = Motions::default();
    motions.retarget(wid(1), A, B, t0, spec(500, Easing::Linear));
    motions.retarget(wid(2), B, A, t0, spec(500, Easing::Linear));

    // Act -- seven frames find window 2 still to handle the one before.
    let early: Vec<_> = (1..=7)
        .map(|i| pending_2(&mut motions, t0, i * 7))
        .collect();
    let late = pending_2(&mut motions, t0, 56);

    // Assert -- seven frames are not enough at 144 Hz; 50 ms are.
    assert!(early.iter().all(|tick| tick.finished.is_empty()));
    assert_eq!(late.finished, [(wid(1), B), (wid(2), A)]);
}

#[test]
fn a_window_sent_elsewhere_joins_the_step_that_sent_it() {
    // Arrange -- window 1 moves in a step of its own, then the next step
    // sends it elsewhere along with window 2.
    let t0 = Instant::now();
    let slow = spec(500, Easing::Linear);
    let mut motions = Motions::default();
    motions.begin_step();
    motions.retarget(wid(1), A, B, t0, slow);
    motions.begin_step();
    motions.retarget(wid(1), A, D, t0 + ms(10), slow);
    motions.retarget(wid(2), B, A, t0 + ms(10), slow);

    // Act -- window 2 falls behind.
    let ticks = [16, 32, 48, 64].map(|at| pending_2(&mut motions, t0, at));

    // Assert -- window 1 ends with it, at its new target.
    let ended: Vec<_> = ticks
        .iter()
        .flat_map(|tick| tick.finished.clone())
        .collect();
    assert!(ended.contains(&(wid(1), D)), "{ended:?}");
}

#[test]
fn a_window_maximized_minimized_or_gone_meanwhile_stops_where_it_is() {
    // Arrange
    let t0 = Instant::now();
    let mut motions = Motions::default();
    for (id, from, to) in [(1, A, B), (2, B, C), (3, C, D), (4, D, A)] {
        motions.retarget(wid(id), from, to, t0, spec(180, Easing::Linear));
    }
    let mut tick = motions.frame(t0 + ms(16));
    let outcomes = [
        FrameOutcome::Maximized,
        FrameOutcome::Minimized,
        FrameOutcome::Gone,
        FrameOutcome::Handled { reached: D },
    ];

    // Act
    motions.take(&mut tick, &outcomes, t0 + ms(16));

    // Assert -- none of them is placed anywhere, and window 4 goes on.
    assert_eq!(tick.stopped, [wid(1), wid(2), wid(3)]);
    assert!(tick.finished.is_empty());
    assert!(motions.is_moving(wid(4)) && !motions.is_moving(wid(1)));
}

#[test]
fn only_the_first_frame_of_a_motion_says_so() {
    // Arrange
    let t0 = Instant::now();
    let linear = spec(180, Easing::Linear);
    let mut motions = Motions::default();
    motions.retarget(wid(1), A, B, t0, linear);

    // Act -- one frame, then a new target mid-flight.
    let first = motions.frame(t0 + ms(16));
    motions.retarget(wid(1), A, C, t0 + ms(20), linear);
    let turned = motions.frame(t0 + ms(32));

    // Assert -- the platform reads the window again only as it starts.
    assert!(first.in_flight[0].first);
    assert!(!turned.in_flight[0].first);
}
