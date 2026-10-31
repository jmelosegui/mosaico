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

fn behind(window: u64, other: u64) -> Effect {
    Effect::PutBehind {
        window: wid(window),
        other: wid(other),
    }
}

#[test]
fn a_window_taking_focus_goes_behind_another_after_the_focus_lands() {
    // Arrange -- taking focus brings a window to the front, so putting
    // it behind window 3 first would be undone.
    let t0 = Instant::now();
    let mut c = Coalescer::default();
    c.admit(focus(2), t0);

    // Act
    let now = c.admit(behind(2, 3), t0);
    let other = c.admit(behind(1, 3), t0);

    // Assert -- only the window taking focus waits.
    assert_eq!(now, None);
    assert_eq!(other, Some(behind(1, 3)));
    assert_eq!(c.take(), [focus(2), behind(2, 3)]);
}

#[test]
fn a_restack_held_for_the_focus_is_forgotten_with_it() {
    // Arrange
    let t0 = Instant::now();
    let mut c = Coalescer::default();
    c.admit(focus(2), t0);
    c.admit(behind(2, 3), t0);

    // Act -- the window leaving is the one taking focus, not the one
    // it was to go behind.
    c.forget(wid(3));
    c.forget(wid(2));

    // Assert
    assert_eq!(c.take(), []);
    assert_eq!(c.held_since(), None);
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
