//! The style a border on screen was last drawn with.

use super::BorderStyle;

/// The style a platform last drew a border with, for platforms where
/// drawing a border costs much more than putting it back in place.
///
/// A platform keeps one per border. When the tiler asks for the border
/// again in a style that draws the same picture, the drawing can be
/// skipped, and only putting the border back behind its window, at its
/// rect, is left to do. That holds only while the border still shows
/// what was drawn: a platform whose border may have lost it starts a new
/// `LastDrawn` for it.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct LastDrawn(Option<BorderStyle>);

impl LastDrawn {
    /// Whether showing `style` needs the border drawn: the picture it
    /// draws is not the one the border was last drawn with.
    ///
    /// The picture depends on the size of the rect, the color, the
    /// stroke width and the radius. Where the rect is does not count:
    /// putting the border in place moves it there.
    pub fn needs_render(&self, style: &BorderStyle) -> bool {
        self.0
            .as_ref()
            .is_none_or(|drawn| !same_picture(drawn, style))
    }

    /// Records how drawing `style` went. A drawing that failed leaves the
    /// border showing something else, so the next one draws again.
    pub fn rendered(&mut self, style: &BorderStyle, done: bool) {
        self.0 = done.then(|| style.clone());
    }
}

/// Whether `a` and `b` draw the same picture, wherever it goes.
fn same_picture(a: &BorderStyle, b: &BorderStyle) -> bool {
    a.rect.width == b.rect.width
        && a.rect.height == b.rect.height
        && a.color == b.color
        && a.width == b.width
        && a.radius == b.radius
}

#[cfg(test)]
#[path = "tests/last_drawn_tests.rs"]
mod tests;
