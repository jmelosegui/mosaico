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
        behind: None,
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
        let overlay = Overlay {
            border,
            drawn,
            width: 4,
        };
        decorations.borders.insert(WindowId::from_raw(raw), overlay);
    }

    // Act
    decorations.forget_drawn();

    // Assert
    for overlay in decorations.borders.values() {
        assert!(overlay.drawn.needs_render(&style()));
    }
}

#[test]
fn moving_a_border_keeps_what_it_was_drawn_with() {
    // Arrange -- an overlay of our own, never shown, whose bitmap shows
    // the style. Without a desktop to make it on, skip.
    let mut decorations = Win32Decorations::default();
    let Ok(border) = Border::new() else {
        return;
    };
    let mut drawn = LastDrawn::default();
    drawn.rendered(&style(), true);
    let window = WindowId::from_raw(1);
    let overlay = Overlay {
        border,
        drawn: drawn.clone(),
        width: 4,
    };
    decorations.borders.insert(window, overlay);
    let elsewhere = Rect::new(-19000, -20000, 40, 30);

    // Act
    let moved = decorations.move_border(window, elsewhere);
    let without = decorations.move_border(WindowId::from_raw(2), elsewhere);

    // Assert -- and a window without a border is no error.
    assert!(moved.is_ok() && without.is_ok());
    assert_eq!(decorations.borders[&window].drawn, drawn);
}

#[test]
fn forgetting_how_borders_were_drawn_renders_the_preview_again_too() {
    // Arrange -- a preview of our own, never shown, whose bitmap shows the
    // style. Without a desktop to make it on, skip.
    let Ok(mut preview) = Preview::new() else {
        return;
    };
    let rect = Rect::new(-20000, -20000, 40, 30);
    let style = PreviewStyle {
        color: style().color,
        width: 4,
        radius: 8,
    };
    preview.rendered_for(rect, &style);
    let mut decorations = Win32Decorations {
        preview: Some(preview),
        ..Win32Decorations::default()
    };

    // Act
    decorations.forget_drawn();

    // Assert
    let preview = decorations.preview.as_ref().expect("the preview is kept");
    assert!(preview.needs_render(rect, &style));
}
