//! The drop preview's effects, carried out against a platform that
//! records what it is asked to draw, and the default of a platform that
//! draws none.

use crate::config::{Color, CornerStyle};
use crate::id::wid;
use crate::tiler::{BorderStyle, Effect, PreviewStyle};
use crate::{Decorations, Error, Rect, Result, WindowId};

use super::apply_decoration_effect;

/// A platform that draws borders and corners but no preview, recording
/// nothing.
struct NoPreview;

impl Decorations for NoPreview {
    fn show_border(&mut self, _: WindowId, _: &BorderStyle) -> Result<()> {
        Ok(())
    }

    fn hide_border(&mut self, _: WindowId) -> Result<()> {
        Ok(())
    }

    fn drop_border(&mut self, _: WindowId) -> Result<()> {
        Ok(())
    }

    fn forget_drawn(&mut self) {}

    fn set_corners(&mut self, _: WindowId, _: CornerStyle) -> Result<()> {
        Ok(())
    }

    fn reset_corners(&mut self, _: WindowId) -> Result<()> {
        Ok(())
    }
}

/// A platform that records the previews it is asked to show and hide.
#[derive(Default)]
struct Previews {
    shown: Vec<(WindowId, Rect, PreviewStyle)>,
    hidden: usize,
}

impl Decorations for Previews {
    fn show_border(&mut self, _: WindowId, _: &BorderStyle) -> Result<()> {
        Ok(())
    }

    fn hide_border(&mut self, _: WindowId) -> Result<()> {
        Ok(())
    }

    fn drop_border(&mut self, _: WindowId) -> Result<()> {
        Ok(())
    }

    fn forget_drawn(&mut self) {}

    fn set_corners(&mut self, _: WindowId, _: CornerStyle) -> Result<()> {
        Ok(())
    }

    fn reset_corners(&mut self, _: WindowId) -> Result<()> {
        Ok(())
    }

    fn show_preview(&mut self, window: WindowId, rect: Rect, style: &PreviewStyle) -> Result<()> {
        self.shown.push((window, rect, style.clone()));
        Ok(())
    }

    fn hide_preview(&mut self) -> Result<()> {
        self.hidden += 1;
        Ok(())
    }
}

fn style() -> PreviewStyle {
    PreviewStyle {
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
fn the_preview_effects_show_and_hide_the_preview() {
    // Arrange
    let mut previews = Previews::default();
    let rect = Rect::new(100, 100, 640, 480);
    let window = wid(1);
    let show = Effect::ShowPreview {
        window,
        rect,
        style: style(),
    };

    // Act
    apply_decoration_effect(&show, &mut previews);
    apply_decoration_effect(&Effect::HidePreview, &mut previews);

    // Assert
    assert_eq!(previews.shown, [(window, rect, style())]);
    assert_eq!(previews.hidden, 1);
}

#[test]
fn a_platform_without_a_preview_says_so_and_has_none_to_hide() {
    // Arrange
    let mut platform = NoPreview;

    // Act
    let shown = platform.show_preview(wid(1), Rect::new(0, 0, 10, 10), &style());
    let hidden = platform.hide_preview();

    // Assert
    assert!(matches!(shown, Err(Error::Unsupported(_))), "{shown:?}");
    assert!(hidden.is_ok());
}
