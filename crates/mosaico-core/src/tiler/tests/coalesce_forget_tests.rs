use std::time::Instant;

use crate::Rect;
use crate::id::wid;

use super::*;

#[test]
fn forgetting_a_destroyed_window_drops_only_what_was_held_for_it() {
    // Arrange -- the platform forgets a window when it is destroyed.
    let t0 = Instant::now();
    let mut c = Coalescer::default();
    let rect = Rect::new(0, 0, 100, 100);
    for window in [wid(1), wid(2)] {
        c.admit(
            Effect::Place {
                window,
                rect,
                timing: Timing::Coalesced,
            },
            t0,
        );
        c.admit(Effect::CenterCursor { window }, t0);
    }

    // Act
    c.forget(wid(1));

    // Assert
    assert_eq!(
        c.take(),
        vec![
            Effect::Place {
                window: wid(2),
                rect,
                timing: Timing::Now,
            },
            Effect::CenterCursor { window: wid(2) },
        ]
    );
}
