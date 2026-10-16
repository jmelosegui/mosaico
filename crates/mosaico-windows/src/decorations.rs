//! Focus borders and window corners, drawn through Win32.

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use mosaico_core::config::CornerStyle;
use mosaico_core::tiler::{BorderStyle, LastDrawn};
use mosaico_core::{Decorations, Rect, Result, WindowId};

use crate::border::Border;
use crate::frame;
use crate::window::Window;

/// [`Decorations`] on the Win32 desktop.
///
/// Each border is a layered overlay window of its own, created the
/// first time its window gets a border and destroyed when it is dropped.
/// The overlays belong to this process, whose events the event hook
/// skips, so drawing them is never reported as a window event. Corners
/// are the window's DWM corner preference, which Windows 10 does not
/// have: there, changing corners is reported as unsupported.
///
/// Rendering a border's bitmap takes tens of milliseconds for a large
/// tile, so each overlay remembers the style it was last rendered with,
/// in a [`LastDrawn`]. A border drawn again the same way, or only moved,
/// is put back behind its window at its rect, keeping its bitmap: a
/// layered window keeps the bitmap `UpdateLayeredWindow` gave it when
/// it is moved or hidden. When the tiler asks for every border to be
/// drawn from scratch, each overlay renders its bitmap the next time.
#[derive(Default)]
pub(crate) struct Win32Decorations {
    /// One overlay per bordered window.
    borders: HashMap<WindowId, Overlay>,
}

/// A border overlay, what its bitmap shows, and how wide its stroke was
/// when it was last put in place, which moving it keeps.
struct Overlay {
    border: Border,
    drawn: LastDrawn,
    width: i32,
}

impl Decorations for Win32Decorations {
    /// Fails only when the overlay cannot be created. Drawing it puts it
    /// just behind the window, in the window's topmost band, even when
    /// the bitmap is kept, since the window may have moved in the
    /// z-order. Placing it behind a window that is gone fails without a
    /// trace, and the overlay stays where it was drawn until the tiler
    /// drops it. A bitmap that cannot be rendered is logged for
    /// debugging, and rendered again the next time.
    fn show_border(&mut self, window: WindowId, style: &BorderStyle) -> Result<()> {
        let Overlay {
            border,
            drawn,
            width,
        } = match self.borders.entry(window) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(Overlay {
                border: Border::new()?,
                drawn: LastDrawn::default(),
                width: style.width,
            }),
        };
        *width = style.width;
        let target = Window::from_id(window).hwnd();
        if drawn.needs_render(style) {
            let rendered = border.show(&style.rect, style.color, style.width, style.radius, target);
            drawn.rendered(style, rendered.is_ok());
            if let Err(e) = rendered {
                mosaico_core::log_debug!("border {}: {}", window, e);
            }
        } else {
            border.restack(&style.rect, style.width, target);
        }
        Ok(())
    }

    /// Hiding keeps the bitmap, so showing the border again the same way
    /// only puts it back.
    fn hide_border(&mut self, window: WindowId) -> Result<()> {
        if let Some(overlay) = self.borders.get(&window) {
            overlay.border.hide();
        }
        Ok(())
    }

    /// Only moves the overlay, which keeps its bitmap, its size, its place
    /// in the z-order and whether it shows.
    fn move_border(&mut self, window: WindowId, rect: Rect) -> Result<()> {
        if let Some(overlay) = self.borders.get(&window) {
            overlay.border.move_to(&rect, overlay.width);
        }
        Ok(())
    }

    fn drop_border(&mut self, window: WindowId) -> Result<()> {
        // Dropping destroys the overlay window.
        self.borders.remove(&window);
        Ok(())
    }

    /// Each overlay keeps its bitmap on screen, and renders it again the
    /// next time its border is drawn.
    fn forget_drawn(&mut self) {
        for overlay in self.borders.values_mut() {
            overlay.drawn = LastDrawn::default();
        }
    }

    fn set_corners(&mut self, window: WindowId, corners: CornerStyle) -> Result<()> {
        frame::set_corner_preference(Window::from_id(window).hwnd(), corners)
    }

    fn reset_corners(&mut self, window: WindowId) -> Result<()> {
        frame::reset_corner_preference(Window::from_id(window).hwnd())
    }
}

#[cfg(test)]
#[path = "decorations_tests.rs"]
mod tests;
