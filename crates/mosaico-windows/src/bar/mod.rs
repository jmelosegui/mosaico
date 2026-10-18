//! Status bar overlay windows, one per monitor.
//!
//! Core's [`BarManager`](mosaico_core::bar::BarManager) decides what each
//! bar shows; this module puts it on screen. Each bar is a layered Win32
//! popup that draws its [`BarContent`](mosaico_core::bar::BarContent) with
//! GDI into a DIB and presents it using `UpdateLayeredWindow`. While it
//! takes the pointer, its window procedure reports what the pointer does
//! on it to the bar's [`PointerSink`].

mod canvas;
mod input;
mod input_messages;
mod input_os;
mod pixels;
mod render;
mod surface;
mod text;
pub(crate) mod widgets;
mod window;

use std::rc::Rc;

use mosaico_core::Rect;
use mosaico_core::bar::{BarId, BarRenderer, BarSurface, PointerSink};

use window::Bar;

/// Puts the status bars on screen as layered popup windows.
pub struct Win32BarRenderer;

impl BarRenderer for Win32BarRenderer {
    fn create(
        &mut self,
        id: BarId,
        area: Rect,
        sink: Rc<dyn PointerSink>,
    ) -> mosaico_core::Result<Box<dyn BarSurface>> {
        Ok(Box::new(Bar::new(id, area, sink)?))
    }
}
