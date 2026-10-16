//! Status bar overlay windows, one per monitor.
//!
//! Core's [`BarManager`](mosaico_core::bar::BarManager) decides what each
//! bar shows; this module puts it on screen. Each bar is a layered Win32
//! popup that draws its [`BarContent`](mosaico_core::bar::BarContent) with
//! GDI into a DIB and presents it using `UpdateLayeredWindow`.

mod render;
mod text;
pub(crate) mod widgets;
mod window;

use mosaico_core::Rect;
use mosaico_core::bar::{BarRenderer, BarSurface};

use window::Bar;

/// Puts the status bars on screen as layered popup windows.
pub struct Win32BarRenderer;

impl BarRenderer for Win32BarRenderer {
    fn create(&mut self, area: Rect) -> mosaico_core::Result<Box<dyn BarSurface>> {
        Ok(Box::new(Bar::new(area)?))
    }
}
