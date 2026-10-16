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
                transition: Transition::Snap,
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
                transition: Transition::Snap,
            },
            Effect::CenterCursor { window: wid(2) },
        ]
    );
}

#[test]
fn untiling_a_window_drops_only_what_was_held_for_it() {
    // Arrange -- #135: windows that leave the layout without a border to
    // drop.
    let t0 = Instant::now();
    let mut c = Coalescer::default();
    let rect = Rect::new(0, 0, 100, 100);
    for window in [wid(1), wid(2)] {
        c.admit(
            Effect::Place {
                window,
                rect,
                timing: Timing::Coalesced,
                transition: Transition::Snap,
            },
            t0,
        );
        c.admit(Effect::CenterCursor { window }, t0);
    }

    // Act
    let admitted = c.admit(Effect::Untile { window: wid(1) }, t0);

    // Assert -- it is not held itself.
    assert_eq!(admitted, Some(Effect::Untile { window: wid(1) }));
    assert_eq!(
        c.take(),
        vec![
            Effect::Place {
                window: wid(2),
                rect,
                timing: Timing::Now,
                transition: Transition::Snap,
            },
            Effect::CenterCursor { window: wid(2) },
        ]
    );
}

/// A focus change to `window` that moves the pointer onto it.
fn focus(window: u64) -> Effect {
    Effect::Focus {
        window: wid(window),
        move_cursor: true,
    }
}

#[test]
fn untiling_a_window_drops_the_focus_change_held_for_it() {
    // Arrange
    let t0 = Instant::now();
    let mut c = Coalescer::default();
    c.admit(focus(1), t0);

    // Act
    c.admit(Effect::Untile { window: wid(1) }, t0);

    // Assert -- nothing is held any more.
    assert!(!c.holds_focus());
    assert_eq!(c.held_since(), None);
    assert_eq!(c.take(), Vec::<Effect>::new());
}

#[test]
fn untiling_another_window_keeps_the_focus_change_held() {
    // Arrange
    let t0 = Instant::now();
    let mut c = Coalescer::default();
    c.admit(focus(1), t0);

    // Act
    c.admit(Effect::Untile { window: wid(2) }, t0);

    // Assert
    assert_eq!(c.take(), vec![focus(1)]);
}

#[test]
fn a_placement_made_now_keeps_the_focus_change_held_for_its_window() {
    // Arrange -- a layout that moves the window at once, while focus on it
    // waits for input to go quiet.
    let t0 = Instant::now();
    let mut c = Coalescer::default();
    c.admit(focus(1), t0);

    // Act
    let placed = c.admit(
        Effect::Place {
            window: wid(1),
            rect: Rect::new(0, 0, 100, 100),
            timing: Timing::Now,
            transition: Transition::Snap,
        },
        t0,
    );

    // Assert
    assert!(placed.is_some());
    assert_eq!(c.take(), vec![focus(1)]);
}
