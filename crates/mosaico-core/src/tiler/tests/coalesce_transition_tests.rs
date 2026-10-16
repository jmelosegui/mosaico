//! What the coalescer keeps of whether a held placement may animate.

use std::time::Instant;

use crate::Rect;
use crate::id::wid;
use crate::tiler::{Coalescer, Effect, Timing, Transition};

/// A placement of window `id` at `x`, as `timing` and `transition` say.
fn place(id: u64, x: i32, timing: Timing, transition: Transition) -> Effect {
    Effect::Place {
        window: wid(id),
        rect: Rect::new(x, 0, 600, 400),
        timing,
        transition,
    }
}

#[test]
fn a_held_placement_is_released_marked_the_way_its_step_marked_it() {
    // Arrange
    let now = Instant::now();
    let mut coalescer = Coalescer::default();
    coalescer.admit(place(1, 0, Timing::Coalesced, Transition::Animate), now);
    coalescer.admit(place(2, 600, Timing::Coalesced, Transition::Snap), now);

    // Act
    let released = coalescer.take();

    // Assert
    let expected = vec![
        place(1, 0, Timing::Now, Transition::Animate),
        place(2, 600, Timing::Now, Transition::Snap),
    ];
    assert_eq!(released, expected);
}

#[test]
fn the_latest_placement_of_a_window_is_released_with_its_own_mark() {
    // Arrange -- a press that lets the window animate, then one that does
    // not, as when the second one sends it to another monitor.
    let now = Instant::now();
    let mut coalescer = Coalescer::default();
    coalescer.admit(place(1, 600, Timing::Coalesced, Transition::Animate), now);

    // Act
    coalescer.admit(place(1, 1920, Timing::Coalesced, Transition::Snap), now);

    // Assert
    let latest = place(1, 1920, Timing::Now, Transition::Snap);
    assert_eq!(coalescer.take(), vec![latest]);
}
