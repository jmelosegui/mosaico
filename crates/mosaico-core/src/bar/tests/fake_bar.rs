//! A fake bar, and a sink that drops what bars report.

use crate::config::bar::BarConfig;
use crate::{Rect, Result};

use super::super::{BarContent, BarId, BarSurface, ItemRegion, PointerEvent, PointerSink};
use super::renderer::{BarEvent, FakeRenderer};
use super::{failed, layout};

/// One bar of a [`FakeRenderer`]. It shares the renderer's record, but
/// dropping it does not count as dropping the renderer.
pub(super) struct FakeBar {
    area: Rect,
    renderer: FakeRenderer,
}

impl FakeBar {
    pub(super) fn new(area: Rect, renderer: FakeRenderer) -> Self {
        Self { area, renderer }
    }
}

impl BarSurface for FakeBar {
    fn draw(&mut self, config: &BarConfig, content: &BarContent) -> Result<Vec<ItemRegion>> {
        let fake = &self.renderer;
        if fake.failing_draws.borrow().contains(&self.area) {
            return Err(failed("draw the bar"));
        }
        fake.events.borrow_mut().push(BarEvent::Drawn {
            area: self.area,
            content: content.clone(),
            background: config.colors.background.clone(),
        });
        Ok(layout::regions(content, self.area.width, config.height))
    }

    fn hide(&mut self) -> Result<()> {
        let fake = &self.renderer;
        fake.events.borrow_mut().push(BarEvent::Hidden(self.area));
        Ok(())
    }

    fn set_click_through(&mut self, click_through: bool) -> Result<()> {
        let fake = &self.renderer;
        if fake.failing_switches.borrow().contains(&self.area) {
            return Err(failed("switch the bar"));
        }
        fake.switches.borrow_mut().push((self.area, click_through));
        Ok(())
    }
}

impl Drop for FakeBar {
    fn drop(&mut self) {
        let fake = &self.renderer;
        fake.events.borrow_mut().push(BarEvent::Dropped(self.area));
    }
}

/// A sink that drops what the bars report: the manager is handed the
/// pointer's input directly in the tests.
pub(crate) struct NoPointer;

impl PointerSink for NoPointer {
    fn send(&self, _bar: BarId, _event: PointerEvent) {}
}
