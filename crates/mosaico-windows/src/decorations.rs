//! Focus borders and window corners, drawn through Win32.

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use mosaico_core::config::CornerStyle;
use mosaico_core::tiler::BorderStyle;
use mosaico_core::{Decorations, Result, WindowId};

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
/// have.
#[derive(Default)]
pub(crate) struct Win32Decorations {
    /// One overlay per bordered window.
    borders: HashMap<WindowId, Border>,
}

impl Decorations for Win32Decorations {
    /// Fails only when the overlay cannot be created. Drawing it puts it
    /// just behind the window, in the window's topmost band. Placing it
    /// behind a window that is gone fails without a trace, and the
    /// overlay stays where it was drawn until the tiler drops it.
    fn show_border(&mut self, window: WindowId, style: &BorderStyle) -> Result<()> {
        let border = match self.borders.entry(window) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(Border::new()?),
        };
        border.show(
            &style.rect,
            style.color,
            style.width,
            style.radius,
            Window::from_id(window).hwnd(),
        );
        Ok(())
    }

    fn hide_border(&mut self, window: WindowId) -> Result<()> {
        if let Some(border) = self.borders.get(&window) {
            border.hide();
        }
        Ok(())
    }

    fn drop_border(&mut self, window: WindowId) -> Result<()> {
        // Dropping destroys the overlay window.
        self.borders.remove(&window);
        Ok(())
    }

    fn set_corners(&mut self, window: WindowId, corners: CornerStyle) -> Result<()> {
        frame::set_corner_preference(Window::from_id(window).hwnd(), corners)
    }

    fn reset_corners(&mut self, window: WindowId) -> Result<()> {
        frame::reset_corner_preference(Window::from_id(window).hwnd())
    }
}
