//! Keeping one border per window in a step.

use crate::Rect;
use crate::config::Color;
use crate::id::wid;
use crate::tiler::{BorderStyle, Timing};

use super::*;

fn place(window: u64, x: i32) -> Effect {
    Effect::Place {
        window: wid(window),
        rect: Rect::new(x, 0, 100, 100),
        timing: Timing::Now,
    }
}

/// The border of `window` drawn around the slot at `x`.
fn draw(window: u64, x: i32) -> Effect {
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

fn hide(window: u64) -> Effect {
    Effect::Border {
        window: wid(window),
        style: None,
    }
}

fn drop_border(window: u64) -> Effect {
    Effect::DropBorder {
        window: wid(window),
    }
}

#[test]
fn each_border_is_drawn_once_right_after_its_window_is_placed() {
    // Arrange -- as a layout, a border refresh and a focus change emit
    // them.
    let effects = vec![
        place(1, 0),
        draw(1, 0),
        place(2, 100),
        draw(1, 0),
        draw(2, 100),
        draw(1, 0),
    ];

    // Act
    let kept = keep_last_borders(effects);

    // Assert
    assert_eq!(kept, [place(1, 0), draw(1, 0), place(2, 100), draw(2, 100)]);
}

#[test]
fn the_last_style_is_drawn_where_the_first_border_after_the_placement_was() {
    // Arrange -- the window loses focus later in the step.
    let mut unfocused = draw(1, 0);
    if let Effect::Border {
        style: Some(style), ..
    } = &mut unfocused
    {
        style.color.r = 0x99;
    }
    let effects = vec![place(1, 0), draw(1, 0), place(2, 100), unfocused.clone()];

    // Act
    let kept = keep_last_borders(effects);

    // Assert
    assert_eq!(kept, [place(1, 0), unfocused, place(2, 100)]);
}

#[test]
fn the_border_drawn_is_the_one_for_the_slot_the_window_ends_in() {
    // Arrange -- placed twice in the step.
    let effects = vec![place(1, 0), draw(1, 0), place(1, 200), draw(1, 200)];

    // Act
    let kept = keep_last_borders(effects);

    // Assert
    assert_eq!(kept, [place(1, 0), place(1, 200), draw(1, 200)]);
}

#[test]
fn hiding_a_border_wins_over_an_earlier_draw() {
    // Arrange
    let effects = vec![draw(1, 0), hide(1)];

    // Act
    let kept = keep_last_borders(effects);

    // Assert
    assert_eq!(kept, [hide(1)]);
}

#[test]
fn drawing_a_border_wins_over_an_earlier_hide() {
    // Arrange
    let effects = vec![hide(1), draw(1, 0)];

    // Act
    let kept = keep_last_borders(effects);

    // Assert
    assert_eq!(kept, [draw(1, 0)]);
}

#[test]
fn dropping_a_border_wins_over_an_earlier_draw() {
    // Arrange
    let effects = vec![draw(1, 0), drop_border(1)];

    // Act
    let kept = keep_last_borders(effects);

    // Assert
    assert_eq!(kept, [drop_border(1)]);
}

#[test]
fn a_border_drawn_after_it_was_dropped_is_kept() {
    // Arrange -- the platform makes a new overlay for it.
    let effects = vec![drop_border(1), draw(1, 0)];

    // Act
    let kept = keep_last_borders(effects);

    // Assert
    assert_eq!(kept, [drop_border(1), draw(1, 0)]);
}

#[test]
fn a_border_decided_after_a_redraw_is_drawn_after_it() {
    // Arrange -- so it is drawn from scratch.
    let effects = vec![draw(1, 0), Effect::RedrawBorders, draw(1, 0)];

    // Act
    let kept = keep_last_borders(effects);

    // Assert
    assert_eq!(kept, [Effect::RedrawBorders, draw(1, 0)]);
}

#[test]
fn a_redraw_leaves_a_border_decided_before_it_where_it_was() {
    // Arrange
    let effects = vec![place(1, 0), draw(1, 0), Effect::RedrawBorders];

    // Act
    let kept = keep_last_borders(effects);

    // Assert
    assert_eq!(kept, [place(1, 0), draw(1, 0), Effect::RedrawBorders]);
}

#[test]
fn a_border_stays_right_after_its_window_past_a_window_put_behind_it() {
    // Arrange -- the monocle layout puts a waiting window behind the
    // monocle window and places it, then the borders are brought in line.
    let behind = Effect::PutBehind {
        window: wid(2),
        other: wid(1),
    };
    let effects = vec![
        place(1, 0),
        draw(1, 0),
        behind.clone(),
        place(2, 0),
        draw(1, 0),
    ];

    // Act
    let kept = keep_last_borders(effects);

    // Assert -- putting a window behind is passed over like the other
    // window effects.
    assert_eq!(kept, [place(1, 0), draw(1, 0), behind, place(2, 0)]);
}
