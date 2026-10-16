use std::time::Instant;

use crate::Rect;

use crate::id::wid;

use super::*;

fn place(window: u64, x: i32, timing: Timing) -> Effect {
    Effect::snap(wid(window), Rect::new(x, 0, 100, 100), timing)
}

fn focus(window: u64) -> Effect {
    Effect::Focus {
        window: wid(window),
        move_cursor: false,
    }
}

fn border(window: u64) -> Effect {
    Effect::Border {
        window: wid(window),
        style: None,
    }
}

#[test]
fn immediate_effects_pass_through() {
    // Arrange
    let t0 = Instant::now();
    let mut c = Coalescer::default();

    // Act / Assert
    assert_eq!(
        c.admit(place(1, 0, Timing::Now), t0),
        Some(place(1, 0, Timing::Now))
    );
    assert_eq!(c.admit(border(1), t0), Some(border(1)));
    assert!(c.is_empty());
}

#[test]
fn only_the_last_focus_change_is_kept() {
    // Arrange
    let t0 = Instant::now();
    let mut c = Coalescer::default();

    // Act
    assert_eq!(c.admit(focus(1), t0), None);
    assert_eq!(c.admit(focus(2), t0), None);

    // Assert
    assert!(!c.is_empty());
    assert_eq!(c.take(), vec![focus(2)]);
    assert!(c.is_empty());
}

#[test]
fn coalesced_placements_keep_the_latest_rect_per_window() {
    // Arrange
    let t0 = Instant::now();
    let mut c = Coalescer::default();

    // Act
    c.admit(place(1, 0, Timing::Coalesced), t0);
    c.admit(place(2, 10, Timing::Coalesced), t0);
    c.admit(place(1, 20, Timing::Coalesced), t0);

    // Assert
    assert_eq!(
        c.take(),
        vec![place(1, 20, Timing::Now), place(2, 10, Timing::Now)]
    );
}

#[test]
fn borders_wait_for_their_window_and_land_before_focus() {
    // Arrange
    let t0 = Instant::now();
    let mut c = Coalescer::default();
    c.admit(place(1, 0, Timing::Coalesced), t0);

    // Act
    let held = c.admit(border(1), t0);
    let other = c.admit(border(2), t0);
    c.admit(focus(1), t0);

    // Assert
    assert_eq!(held, None);
    assert_eq!(other, Some(border(2)), "window 2 has nothing pending");
    assert_eq!(
        c.take(),
        vec![place(1, 0, Timing::Now), border(1), focus(1)]
    );
}

#[test]
fn an_immediate_placement_supersedes_a_held_one() {
    // Arrange -- a move, then a workspace switch before the flush.
    let t0 = Instant::now();
    let mut c = Coalescer::default();
    c.admit(place(1, 0, Timing::Coalesced), t0);
    c.admit(border(1), t0);

    // Act
    let now = c.admit(place(1, 50, Timing::Now), t0);

    // Assert -- nothing left to move the window back later.
    assert_eq!(now, Some(place(1, 50, Timing::Now)));
    assert!(c.take().is_empty());
}

/// Holds a move of window 1 with its border and cursor, and a move of
/// window 2, as a move across a workspace leaves them.
fn holding_two_moves(t0: Instant) -> Coalescer {
    let mut c = Coalescer::default();
    c.admit(place(1, 0, Timing::Coalesced), t0);
    c.admit(border(1), t0);
    c.admit(Effect::CenterCursor { window: wid(1) }, t0);
    c.admit(place(2, 100, Timing::Coalesced), t0);
    c
}

/// Asserts that `leaving` passes straight through and that nothing held
/// for window 1 is released afterwards, while window 2's move still is.
fn assert_leaving_drops_what_was_held(leaving: Effect) {
    // Arrange
    let t0 = Instant::now();
    let mut c = holding_two_moves(t0);

    // Act
    let passed = c.admit(leaving.clone(), t0);

    // Assert
    assert_eq!(passed, Some(leaving));
    assert_eq!(c.take(), vec![place(2, 100, Timing::Now)]);
}

#[test]
fn hiding_a_window_drops_its_held_move_and_border() {
    // #71
    assert_leaving_drops_what_was_held(Effect::Hide {
        window: wid(1),
        how: crate::config::HidingBehaviour::Cloak,
    });
}

#[test]
fn minimizing_a_window_drops_its_held_move_and_border() {
    // #71
    assert_leaving_drops_what_was_held(Effect::Minimize { window: wid(1) });
}

#[test]
fn closing_a_window_drops_its_held_move_and_border() {
    // #71
    assert_leaving_drops_what_was_held(Effect::Close { window: wid(1) });
}

#[test]
fn dropping_a_border_drops_the_held_move_and_border_too() {
    // #71 -- the overlay is gone, so drawing the held border would make
    // a new one that nothing removes.
    assert_leaving_drops_what_was_held(Effect::DropBorder { window: wid(1) });
}

#[test]
fn a_window_leaving_does_not_drop_another_windows_held_border() {
    // Arrange -- the near miss: only window 1 leaves.
    let t0 = Instant::now();
    let mut c = Coalescer::default();
    c.admit(place(1, 0, Timing::Coalesced), t0);
    c.admit(place(2, 100, Timing::Coalesced), t0);
    c.admit(border(2), t0);

    // Act
    c.admit(Effect::DropBorder { window: wid(1) }, t0);

    // Assert
    assert_eq!(c.take(), vec![place(2, 100, Timing::Now), border(2)]);
}
