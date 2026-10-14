use mosaico_core::Rect;
use mosaico_core::config::Color;

use super::*;

/// A border far off every monitor.
fn style() -> BorderStyle {
    BorderStyle {
        rect: Rect::new(-20000, -20000, 40, 30),
        color: Color {
            r: 0x11,
            g: 0x22,
            b: 0x33,
        },
        width: 4,
        radius: 8,
    }
}

#[test]
fn forgetting_how_borders_were_drawn_renders_every_one_again() {
    // Arrange -- two overlays of our own, never shown, whose bitmaps
    // show the style. Without a desktop to make them on, skip.
    let mut decorations = Win32Decorations::default();
    for raw in [1, 2] {
        let Ok(border) = Border::new() else {
            return;
        };
        let mut drawn = LastDrawn::default();
        drawn.rendered(&style(), true);
        let overlay = Overlay { border, drawn };
        decorations.borders.insert(WindowId::from_raw(raw), overlay);
    }

    // Act
    decorations.forget_drawn();

    // Assert
    for overlay in decorations.borders.values() {
        assert!(overlay.drawn.needs_render(&style()));
    }
}
