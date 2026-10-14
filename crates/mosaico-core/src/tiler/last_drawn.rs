//! The style a border on screen was last drawn with.

use super::BorderStyle;

/// The style a platform last drew a border with, for platforms where
/// drawing a border costs much more than putting it back in place.
///
/// A platform keeps one per border. When the tiler asks for the border
/// again in the same style, the drawing can be skipped, and only putting
/// the border back behind its window, which also shows it when it was
/// hidden, is left to do.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct LastDrawn(Option<BorderStyle>);

impl LastDrawn {
    /// Whether showing `style` needs the border drawn: it is not the
    /// style the border was last drawn with.
    pub fn needs_render(&self, style: &BorderStyle) -> bool {
        self.0.as_ref() != Some(style)
    }

    /// Records how drawing `style` went. A drawing that failed leaves the
    /// border showing something else, so the next one draws again.
    pub fn rendered(&mut self, style: &BorderStyle, done: bool) {
        self.0 = done.then(|| style.clone());
    }
}

#[cfg(test)]
#[path = "tests/last_drawn_tests.rs"]
mod tests;
