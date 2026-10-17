use mosaico_core::config::Color;

use crate::window::Window;

use super::*;

impl Preview {
    /// Takes the bitmap as rendered for `rect` and `style`, as `show` does
    /// once it rendered it, without showing anything.
    pub(crate) fn rendered_for(&mut self, rect: Rect, style: &PreviewStyle) {
        self.drawn = Some((rect, style.clone()));
    }
}

/// How a preview far off every monitor is drawn.
fn style() -> PreviewStyle {
    let color = Color {
        r: 0x11,
        g: 0x22,
        b: 0x33,
    };
    PreviewStyle {
        color,
        width: 4,
        radius: 8,
    }
}

#[test]
fn the_outline_of_a_preview_lies_inside_its_rect() {
    // Act
    let around = inside(Rect::new(100, 50, 640, 480), 4);

    // Assert -- a stroke 4 wide drawn around it covers the rect exactly.
    assert_eq!(around, Rect::new(104, 54, 632, 472));
}

#[test]
fn a_preview_too_small_for_its_outline_keeps_a_pixel_inside() {
    // Act -- and a width below 0 counts as none.
    let tiny = inside(Rect::new(10, 10, 6, 6), 4);
    let none = inside(Rect::new(10, 10, 6, 6), -3);

    // Assert
    assert_eq!(tiny, Rect::new(14, 14, 1, 1));
    assert_eq!(none, Rect::new(10, 10, 6, 6));
}

#[test]
fn a_new_preview_is_hidden_with_a_fill_and_nothing_drawn() {
    // Arrange -- an overlay of our own, never shown. Without a desktop to
    // make it on, skip.
    let Ok(preview) = Preview::new() else {
        return;
    };

    // Act -- hiding one that never showed changes nothing.
    preview.hide();

    // Assert -- with a fill, unlike a border's.
    assert!(!Window::new(preview.overlay.hwnd).is_visible());
    assert_eq!(preview.overlay.fill, FILL_ALPHA);
    assert!(preview.drawn.is_none());
}

#[test]
fn a_preview_renders_its_bitmap_again_once_it_forgot_it() {
    // Arrange -- an overlay of our own, never shown, whose bitmap shows
    // the style. Without a desktop to make it on, skip.
    let Ok(mut preview) = Preview::new() else {
        return;
    };
    let rect = Rect::new(-20000, -20000, 40, 30);
    preview.rendered_for(rect, &style());
    let kept = !preview.needs_render(rect, &style());

    // Act
    preview.forget();

    // Assert
    assert!(kept);
    assert!(preview.needs_render(rect, &style()));
}
