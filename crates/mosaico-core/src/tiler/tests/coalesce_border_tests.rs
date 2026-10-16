//! Borders held until their window's placement is applied.

use std::time::Instant;

use crate::Rect;
use crate::config::Color;
use crate::id::wid;
use crate::tiler::BorderStyle;

use super::*;

/// A move of `window` to the slot at `x`, held until input goes quiet.
fn move_to(window: u64, x: i32) -> Effect {
    Effect::Place {
        window: wid(window),
        rect: Rect::new(x, 0, 100, 100),
        timing: Timing::Coalesced,
        transition: Transition::Snap,
    }
}

/// The same move, as the coalescer releases it.
fn placed(window: u64, x: i32) -> Effect {
    Effect::Place {
        window: wid(window),
        rect: Rect::new(x, 0, 100, 100),
        timing: Timing::Now,
        transition: Transition::Snap,
    }
}

/// The border of `window` around the slot at `x`.
fn border_at(window: u64, x: i32) -> Effect {
    Effect::Border {
        window: wid(window),
        style: Some(BorderStyle {
            rect: Rect::new(x, 0, 100, 100),
            color: Color {
                r: 0x11,
                g: 0x22,
                b: 0x33,
            },
            width: 4,
            radius: 8,
        }),
    }
}

#[test]
fn a_burst_of_moves_draws_the_border_once_around_the_slot_it_lands_in() {
    // Arrange -- two moves of window 1 before input goes quiet.
    let t0 = Instant::now();
    let mut c = Coalescer::default();
    c.admit(move_to(1, 0), t0);
    c.admit(border_at(1, 0), t0);
    c.admit(move_to(1, 200), t0);
    c.admit(border_at(1, 200), t0);

    // Act
    let released = c.take();

    // Assert
    assert_eq!(released, [placed(1, 200), border_at(1, 200)]);
}

#[test]
fn hiding_a_held_border_wins_over_the_draw_held_before() {
    // Arrange
    let t0 = Instant::now();
    let mut c = Coalescer::default();
    c.admit(move_to(1, 0), t0);
    c.admit(border_at(1, 0), t0);
    let hide = Effect::Border {
        window: wid(1),
        style: None,
    };
    c.admit(hide.clone(), t0);

    // Act
    let released = c.take();

    // Assert
    assert_eq!(released, [placed(1, 0), hide]);
}

#[test]
fn each_window_keeps_its_own_held_border_and_cursor_move() {
    // Arrange -- the next step decides window 1's border again.
    let t0 = Instant::now();
    let mut c = Coalescer::default();
    c.admit(move_to(1, 0), t0);
    c.admit(move_to(2, 200), t0);
    c.admit(border_at(1, 0), t0);
    c.admit(Effect::CenterCursor { window: wid(1) }, t0);
    c.admit(border_at(2, 200), t0);
    c.admit(border_at(1, 0), t0);

    // Act
    let released = c.take();

    // Assert
    assert_eq!(
        released,
        [
            placed(1, 0),
            placed(2, 200),
            border_at(1, 0),
            Effect::CenterCursor { window: wid(1) },
            border_at(2, 200),
        ]
    );
}

#[test]
fn a_redraw_of_every_border_is_not_held_behind_a_move() {
    // Arrange
    let t0 = Instant::now();
    let mut c = Coalescer::default();
    c.admit(move_to(1, 0), t0);
    c.admit(border_at(1, 0), t0);

    // Act
    let admitted = c.admit(Effect::RedrawBorders, t0);

    // Assert -- so the border held behind the move is drawn after it.
    assert_eq!(admitted, Some(Effect::RedrawBorders));
    assert_eq!(c.take(), [placed(1, 0), border_at(1, 0)]);
}
