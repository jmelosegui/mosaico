use crate::Rect;

use super::*;

fn place(window: usize, x: i32, timing: Timing) -> Effect {
    Effect::Place {
        window,
        rect: Rect::new(x, 0, 100, 100),
        timing,
    }
}

fn focus(window: usize) -> Effect {
    Effect::Focus {
        window,
        move_cursor: false,
    }
}

fn border(window: usize) -> Effect {
    Effect::Border {
        window,
        style: None,
    }
}

#[test]
fn immediate_effects_pass_through() {
    // Arrange
    let mut c = Coalescer::default();

    // Act / Assert
    assert_eq!(
        c.admit(place(1, 0, Timing::Now)),
        Some(place(1, 0, Timing::Now))
    );
    assert_eq!(c.admit(border(1)), Some(border(1)));
    assert!(c.is_empty());
}

#[test]
fn only_the_last_focus_change_is_kept() {
    // Arrange
    let mut c = Coalescer::default();

    // Act
    assert_eq!(c.admit(focus(1)), None);
    assert_eq!(c.admit(focus(2)), None);

    // Assert
    assert!(!c.is_empty());
    assert_eq!(c.take(), vec![focus(2)]);
    assert!(c.is_empty());
}

#[test]
fn coalesced_placements_keep_the_latest_rect_per_window() {
    // Arrange
    let mut c = Coalescer::default();

    // Act
    c.admit(place(1, 0, Timing::Coalesced));
    c.admit(place(2, 10, Timing::Coalesced));
    c.admit(place(1, 20, Timing::Coalesced));

    // Assert
    assert_eq!(
        c.take(),
        vec![place(1, 20, Timing::Now), place(2, 10, Timing::Now)]
    );
}

#[test]
fn borders_wait_for_their_window_and_land_before_focus() {
    // Arrange
    let mut c = Coalescer::default();
    c.admit(place(1, 0, Timing::Coalesced));

    // Act
    let held = c.admit(border(1));
    let other = c.admit(border(2));
    c.admit(focus(1));

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
    let mut c = Coalescer::default();
    c.admit(place(1, 0, Timing::Coalesced));
    c.admit(border(1));

    // Act
    let now = c.admit(place(1, 50, Timing::Now));

    // Assert -- nothing left to move the window back later.
    assert_eq!(now, Some(place(1, 50, Timing::Now)));
    assert!(c.take().is_empty());
}
