//! A renderer of fake bars, and what happens to them.

use std::cell::RefCell;
use std::rc::Rc;

use crate::{Rect, Result};

use super::super::{BarContent, BarId, BarRenderer, BarSurface, PointerSink};
use super::bar::FakeBar;
use super::failed;

/// What happened to a [`FakeRenderer`]'s bars, in order.
#[derive(Debug, Clone, PartialEq)]
pub(in crate::bar) enum BarEvent {
    /// A bar was made for this area.
    Created(Rect),
    /// The bar for `area` was drawn with `content`, on `background`.
    Drawn {
        area: Rect,
        content: BarContent,
        background: String,
    },
    /// The bar for this area was hidden.
    Hidden(Rect),
    /// The bar for this area was dropped, which removes it.
    Dropped(Rect),
    /// The renderer the code under test owned was dropped.
    RendererDropped,
}

/// Bars that only record what happens to them, and draw every item at
/// a fixed width (see `fake_layout.rs`).
///
/// Clones share one record, so a test can keep a clone while the code
/// under test owns another.
#[derive(Debug, Clone, Default)]
pub(in crate::bar) struct FakeRenderer {
    pub(super) events: Rc<RefCell<Vec<BarEvent>>>,
    /// Areas whose bar cannot be made.
    failing: Rc<RefCell<Vec<Rect>>>,
    /// Areas whose bar cannot be drawn.
    pub(super) failing_draws: Rc<RefCell<Vec<Rect>>>,
    /// The id each bar was made with, in order.
    ids: Rc<RefCell<Vec<BarId>>>,
    /// Each switch a bar was asked for, by area: `true` to let the pointer
    /// through. Kept apart from `events`, which the tests of drawing read.
    pub(super) switches: Rc<RefCell<Vec<(Rect, bool)>>>,
    /// Areas whose bar cannot be switched.
    pub(super) failing_switches: Rc<RefCell<Vec<Rect>>>,
}

impl FakeRenderer {
    /// Makes creating the bar for `area` fail.
    pub(in crate::bar) fn fail_on(&self, area: Rect) {
        self.failing.borrow_mut().push(area);
    }

    /// Makes drawing the bar for `area` fail, until [`heal_draws`](Self::heal_draws).
    pub(in crate::bar) fn fail_draws_on(&self, area: Rect) {
        self.failing_draws.borrow_mut().push(area);
    }

    /// Lets every bar draw again.
    pub(in crate::bar) fn heal_draws(&self) {
        self.failing_draws.borrow_mut().clear();
    }

    /// What happened since the last call.
    pub(in crate::bar) fn take(&self) -> Vec<BarEvent> {
        std::mem::take(&mut self.events.borrow_mut())
    }

    /// The id each bar was made with so far, in order.
    pub(in crate::bar) fn ids(&self) -> Vec<BarId> {
        self.ids.borrow().clone()
    }

    /// The switches between letting the pointer through and taking it
    /// asked for since the last call, by area.
    pub(in crate::bar) fn take_switches(&self) -> Vec<(Rect, bool)> {
        std::mem::take(&mut self.switches.borrow_mut())
    }

    /// Makes switching the bar for `area` fail, until
    /// [`heal_switches`](Self::heal_switches).
    pub(in crate::bar) fn fail_switches_on(&self, area: Rect) {
        self.failing_switches.borrow_mut().push(area);
    }

    /// Lets every bar be switched again.
    pub(in crate::bar) fn heal_switches(&self) {
        self.failing_switches.borrow_mut().clear();
    }

    /// The renderer to hand the code under test. Dropping it is recorded,
    /// unlike dropping the test's own clone.
    pub(in crate::bar) fn boxed(&self) -> Box<dyn BarRenderer> {
        Box::new(OwnedRenderer(self.clone()))
    }
}

/// The copy of a [`FakeRenderer`] that the code under test owns.
struct OwnedRenderer(FakeRenderer);

impl BarRenderer for OwnedRenderer {
    fn create(
        &mut self,
        id: BarId,
        area: Rect,
        _sink: Rc<dyn PointerSink>,
    ) -> Result<Box<dyn BarSurface>> {
        let fake = &self.0;
        if fake.failing.borrow().contains(&area) {
            return Err(failed("create the bar"));
        }
        fake.events.borrow_mut().push(BarEvent::Created(area));
        fake.ids.borrow_mut().push(id);
        Ok(Box::new(FakeBar::new(area, fake.clone())))
    }
}

impl Drop for OwnedRenderer {
    fn drop(&mut self) {
        self.0.events.borrow_mut().push(BarEvent::RendererDropped);
    }
}
