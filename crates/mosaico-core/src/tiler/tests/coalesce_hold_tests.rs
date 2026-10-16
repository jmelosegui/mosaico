use std::time::{Duration, Instant};

use crate::Rect;
use crate::id::wid;

use super::*;

fn moved(window: u64) -> Effect {
    Effect::Place {
        window: wid(window),
        rect: Rect::new(0, 0, 100, 100),
        timing: Timing::Coalesced,
        transition: Transition::Snap,
    }
}

fn focus(window: u64) -> Effect {
    Effect::Focus {
        window: wid(window),
        move_cursor: false,
    }
}

#[test]
fn nothing_held_has_no_hold_time() {
    // Arrange
    let t0 = Instant::now();
    let mut c = Coalescer::default();

    // Act -- effects applied right away start no hold.
    c.admit(Effect::Repaint { window: wid(1) }, t0);

    // Assert
    assert_eq!(c.held_since(), None);
}

#[test]
fn the_hold_starts_with_the_first_held_effect() {
    // Arrange
    let t0 = Instant::now();
    let mut c = Coalescer::default();

    // Act -- later effects join the hold instead of restarting it.
    c.admit(focus(1), t0);
    c.admit(moved(2), t0 + Duration::from_millis(10));
    c.admit(focus(3), t0 + Duration::from_millis(20));

    // Assert
    assert_eq!(c.held_since(), Some(t0));
}

#[test]
fn releasing_what_is_held_ends_the_hold() {
    // Arrange
    let t0 = Instant::now();
    let t1 = t0 + Duration::from_millis(30);
    let mut c = Coalescer::default();
    c.admit(focus(1), t0);

    // Act
    c.take();
    let after_take = c.held_since();
    c.admit(focus(2), t1);

    // Assert -- the next hold starts afresh.
    assert_eq!(after_take, None);
    assert_eq!(c.held_since(), Some(t1));
}

#[test]
fn forgetting_the_last_held_window_ends_the_hold() {
    // Arrange
    let t0 = Instant::now();
    let mut c = Coalescer::default();
    c.admit(moved(1), t0);

    // Act
    c.forget(wid(1));

    // Assert
    assert_eq!(c.held_since(), None);
}
