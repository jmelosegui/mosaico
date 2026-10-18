//! The drop preview: an overlay like a border's, of its own, whose bitmap
//! has a translucent fill inside its outline.
//!
//! It is created hidden the first time it is shown, and like a border it
//! is click-through (`WS_EX_TRANSPARENT`), has no taskbar entry
//! (`WS_EX_TOOLWINDOW`) and belongs to this process, whose events the
//! hooks skip. Showing it puts it just behind the window the user drags,
//! in that window's band, topmost or not: behind that window, and in
//! front of the windows behind it.

use mosaico_core::tiler::PreviewStyle;
use mosaico_core::{Rect, Result};
use windows::Win32::Foundation::HWND;

use super::Border;

/// How opaque the preview's fill is, out of 255: enough to see where the
/// window lands, little enough to see the windows under it.
const FILL_ALPHA: u8 = 64;

/// The drop preview overlay, and what its bitmap shows.
pub(crate) struct Preview {
    overlay: Border,
    /// The rect and style the bitmap was last rendered for, once one was.
    drawn: Option<(Rect, PreviewStyle)>,
}

impl Preview {
    /// Creates the preview, hidden.
    pub(crate) fn new() -> Result<Self> {
        let mut overlay = Border::new()?;
        overlay.fill = FILL_ALPHA;
        Ok(Self {
            overlay,
            drawn: None,
        })
    }

    /// Shows the preview over `rect`, its outline inside it, drawn as
    /// `style` says, just behind `target`, the window the user drags. Its
    /// bitmap is rendered again only when it [`needs_render`]; otherwise
    /// it is only put back in place. Fails when a new bitmap does not
    /// reach the overlay, which is put in place either way and rendered
    /// again the next time.
    ///
    /// [`needs_render`]: Self::needs_render
    pub(crate) fn show(&mut self, rect: Rect, style: &PreviewStyle, target: HWND) -> Result<()> {
        let around = inside(rect, style.width);
        if !self.needs_render(rect, style) {
            self.overlay.restack(&around, style.width, target);
            return Ok(());
        }
        let (color, width, radius) = (style.color, style.width, style.radius);
        let rendered = self.overlay.show(&around, color, width, radius, target);
        self.drawn = rendered.is_ok().then(|| (rect, style.clone()));
        rendered
    }

    /// Whether showing the preview over `rect` as `style` says renders its
    /// bitmap: unless it was last rendered for both, and not forgotten
    /// since.
    pub(crate) fn needs_render(&self, rect: Rect, style: &PreviewStyle) -> bool {
        let same = |(r, s): &(Rect, PreviewStyle)| *r == rect && s == style;
        !self.drawn.as_ref().is_some_and(same)
    }

    /// Forgets what the bitmap was rendered for, as one lost on a display
    /// change may be, so the next `show` renders it again.
    pub(crate) fn forget(&mut self) {
        self.drawn = None;
    }

    /// Hides the preview, keeping its bitmap to show again.
    pub(crate) fn hide(&self) {
        self.overlay.hide();
    }
}

/// `rect` with each side moved in by `width`, so that a stroke `width`
/// wide drawn around it lies inside `rect`. Never less than a pixel wide
/// or tall.
pub(crate) fn inside(rect: Rect, width: i32) -> Rect {
    let width = width.max(0);
    let (w, h) = (rect.width - 2 * width, rect.height - 2 * width);
    Rect::new(rect.x + width, rect.y + width, w.max(1), h.max(1))
}

#[cfg(test)]
#[path = "preview_tests.rs"]
mod tests;
