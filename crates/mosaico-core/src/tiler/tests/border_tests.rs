//! The colors focus borders are drawn in.

use crate::config::{BorderColors, Color};

use super::*;

/// The color of the last border `effects` draw around `window`, or
/// `None` when the last one hides it or there is none.
fn color_of(effects: &[Effect], window: u64) -> Option<Color> {
    let style = effects.iter().rev().find_map(|e| match e {
        Effect::Border { window: w, style } if *w == wid(window) => Some(style),
        _ => None,
    })?;
    style.as_ref().map(|s| s.color)
}

#[test]
fn a_border_color_that_is_not_a_color_is_drawn_in_the_fallback_color() {
    // Arrange -- a name the theme does not know is left as written.
    let mut settings = Settings::with_gap(8);
    settings.borders.colors.focused = "chartreuse".into();
    settings.borders.colors.unfocused = "#222222".into();

    // Act
    let h = Harness::with_settings(settings, 1, &[(1, 0), (2, 0)]);

    // Assert -- 1 has focus, 2 does not.
    assert_eq!(color_of(&h.effects, 1), Some(BorderColors::FALLBACK));
    assert_eq!(
        color_of(&h.effects, 2),
        Some(Color {
            r: 0x22,
            g: 0x22,
            b: 0x22
        })
    );
}

#[test]
fn none_turns_off_unfocused_borders_only() {
    // Arrange
    let mut settings = Settings::with_gap(8);
    settings.borders.colors.focused = BorderColors::NONE.into();
    settings.borders.colors.unfocused = BorderColors::NONE.into();

    // Act
    let h = Harness::with_settings(settings, 1, &[(1, 0), (2, 0)]);

    // Assert -- the focused border is drawn in the fallback color.
    assert_eq!(color_of(&h.effects, 1), Some(BorderColors::FALLBACK));
    assert!(
        !h.effects
            .iter()
            .any(|e| matches!(e, Effect::Border { window, .. } if *window == wid(2))),
        "the unfocused window has a border: {:?}",
        h.effects
    );
}
