//! Clicks on a bar: a button that goes down and up on the same item.

use super::{BarTarget, Button, ItemRegion, PointerEvent, hit};

/// The press a click on one bar starts with, until it ends.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Press {
    /// The button that went down, and the item it went down on.
    down: Option<(Button, BarTarget)>,
}

impl Press {
    /// Takes what the pointer did, hit tested against `regions`, the ones
    /// the bar drew last, and returns the item it clicked, with the
    /// button: the item a button went down on and then up on.
    ///
    /// A press on no item, a release on another item or off every item,
    /// and a release after the pointer left the bar click nothing. A press
    /// replaces the one before it, so of two buttons held together only
    /// the last one pressed can click.
    pub fn take(
        &mut self,
        event: PointerEvent,
        regions: &[ItemRegion],
    ) -> Option<(Button, BarTarget)> {
        match event {
            PointerEvent::Press { button, x, y } => {
                self.down = hit(regions, x, y).map(|region| (button, region.target));
                None
            }
            PointerEvent::Release { button, x, y } => {
                let down = self.down.take()?;
                let up = hit(regions, x, y)?;
                (down == (button, up.target)).then_some(down)
            }
            PointerEvent::Leave => {
                self.down = None;
                None
            }
            PointerEvent::Wheel { .. } => None,
        }
    }
}

#[cfg(test)]
#[path = "tests/press_support.rs"]
mod support;

#[cfg(test)]
#[path = "tests/press_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/press_redraw_tests.rs"]
mod redraw_tests;
