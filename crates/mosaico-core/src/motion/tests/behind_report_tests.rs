//! Which windows a frame step says fell behind.

use std::time::Instant;

use crate::Rect;
use crate::frames::FrameOutcome;
use crate::id::wid;
use crate::motion::Easing;
use crate::motion::test_support::{ms, spec};

use crate::motion::Motions;

const LEFT: Rect = Rect {
    x: 0,
    y: 0,
    width: 800,
    height: 900,
};
const RIGHT: Rect = Rect { x: 800, ..LEFT };

#[test]
fn only_the_window_that_fell_behind_is_reported_and_only_once() {
    // Arrange -- a swap, with window 2 found still to handle its frame,
    // frame after frame, at 60 Hz.
    let t0 = Instant::now();
    let slow = spec(500, Easing::Linear);
    let mut motions = Motions::default();
    motions.begin_step();
    motions.retarget(wid(1), LEFT, RIGHT, t0, slow);
    motions.retarget(wid(2), RIGHT, LEFT, t0, slow);

    // Act -- one frame step more than it takes not to be seen to move for
    // the limit, 50 ms.
    let mut behind = Vec::new();
    for at in [16, 32, 48, 64, 80] {
        let mut tick = motions.frame(t0 + ms(at));
        let found = |window| match window == wid(2) {
            true => FrameOutcome::Pending,
            false => FrameOutcome::Handled { reached: LEFT },
        };
        let outcomes: Vec<_> = tick.in_flight.iter().map(|f| found(f.window)).collect();
        motions.take(&mut tick, &outcomes, t0 + ms(at));
        behind.push(tick.behind);
    }

    // Assert -- window 1 ended with it, but did not fall behind.
    assert!(behind[..3].iter().all(Vec::is_empty), "{behind:?}");
    assert_eq!(behind[3], [wid(2)]);
    assert!(behind[4].is_empty(), "{behind:?}");
}
