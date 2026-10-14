//! Fakes of the bar's platform contracts.

use std::cell::RefCell;
use std::rc::Rc;

use crate::config::bar::BarConfig;
use crate::{Error, Rect, Result};

use super::{BarContent, BarRenderer, BarSurface, CpuTimes, LocalTime, NowPlaying, SystemInfo};

/// What a [`FakeSystem`] reports.
#[derive(Debug, Default)]
pub(super) struct Machine {
    /// Processor counters. `None` makes reading them fail.
    pub(super) cpu: Option<CpuTimes>,
    /// Memory in use. `None` makes reading it fail.
    pub(super) memory: Option<u32>,
    /// What is playing.
    pub(super) playing: Option<NowPlaying>,
    /// Makes reading what is playing fail.
    pub(super) media_fails: bool,
    /// The local time.
    pub(super) time: LocalTime,
    reads: Reads,
}

/// How often each reading of a [`FakeSystem`] was taken.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) struct Reads {
    pub(super) cpu: usize,
    pub(super) memory: usize,
    pub(super) media: usize,
    pub(super) time: usize,
}

/// A machine whose readings the test sets.
///
/// Clones share one [`Machine`], so a test can keep a clone while the
/// code under test owns another.
#[derive(Debug, Clone, Default)]
pub(super) struct FakeSystem(Rc<RefCell<Machine>>);

impl FakeSystem {
    /// Changes what the machine reports.
    pub(super) fn set(&self, change: impl FnOnce(&mut Machine)) {
        change(&mut self.0.borrow_mut());
    }

    /// How often each reading was taken so far.
    pub(super) fn reads(&self) -> Reads {
        self.0.borrow().reads
    }
}

impl SystemInfo for FakeSystem {
    fn cpu_times(&self) -> Result<CpuTimes> {
        let mut machine = self.0.borrow_mut();
        machine.reads.cpu += 1;
        machine
            .cpu
            .ok_or_else(|| failed("read the processor times"))
    }

    fn memory_usage(&self) -> Result<u32> {
        let mut machine = self.0.borrow_mut();
        machine.reads.memory += 1;
        machine.memory.ok_or_else(|| failed("read the memory"))
    }

    fn now_playing(&self) -> Result<Option<NowPlaying>> {
        let mut machine = self.0.borrow_mut();
        machine.reads.media += 1;
        if machine.media_fails {
            return Err(failed("read what is playing"));
        }
        Ok(machine.playing.clone())
    }

    fn local_time(&self) -> LocalTime {
        let mut machine = self.0.borrow_mut();
        machine.reads.time += 1;
        machine.time
    }
}

/// What happened to a [`FakeRenderer`]'s bars, in order.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum BarEvent {
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

/// Bars that only record what happens to them.
///
/// Clones share one record, so a test can keep a clone while the code
/// under test owns another.
#[derive(Debug, Clone, Default)]
pub(super) struct FakeRenderer {
    events: Rc<RefCell<Vec<BarEvent>>>,
    /// Areas whose bar cannot be made.
    failing: Rc<RefCell<Vec<Rect>>>,
    /// Areas whose bar cannot be drawn.
    failing_draws: Rc<RefCell<Vec<Rect>>>,
}

impl FakeRenderer {
    /// Makes creating the bar for `area` fail.
    pub(super) fn fail_on(&self, area: Rect) {
        self.failing.borrow_mut().push(area);
    }

    /// Makes drawing the bar for `area` fail, until [`heal_draws`](Self::heal_draws).
    pub(super) fn fail_draws_on(&self, area: Rect) {
        self.failing_draws.borrow_mut().push(area);
    }

    /// Lets every bar draw again.
    pub(super) fn heal_draws(&self) {
        self.failing_draws.borrow_mut().clear();
    }

    /// What happened since the last call.
    pub(super) fn take(&self) -> Vec<BarEvent> {
        std::mem::take(&mut self.events.borrow_mut())
    }

    /// The renderer to hand the code under test. Dropping it is recorded,
    /// unlike dropping the test's own clone.
    pub(super) fn boxed(&self) -> Box<dyn BarRenderer> {
        Box::new(OwnedRenderer(self.clone()))
    }
}

/// The copy of a [`FakeRenderer`] that the code under test owns.
struct OwnedRenderer(FakeRenderer);

impl BarRenderer for OwnedRenderer {
    fn create(&mut self, area: Rect) -> Result<Box<dyn BarSurface>> {
        let fake = &self.0;
        if fake.failing.borrow().contains(&area) {
            return Err(failed("create the bar"));
        }
        fake.events.borrow_mut().push(BarEvent::Created(area));
        Ok(Box::new(FakeBar {
            area,
            events: Rc::clone(&fake.events),
            failing_draws: Rc::clone(&fake.failing_draws),
        }))
    }
}

impl Drop for OwnedRenderer {
    fn drop(&mut self) {
        self.0.events.borrow_mut().push(BarEvent::RendererDropped);
    }
}

/// One bar of a [`FakeRenderer`].
struct FakeBar {
    area: Rect,
    events: Rc<RefCell<Vec<BarEvent>>>,
    failing_draws: Rc<RefCell<Vec<Rect>>>,
}

impl BarSurface for FakeBar {
    fn draw(&mut self, config: &BarConfig, content: &BarContent) -> Result<()> {
        if self.failing_draws.borrow().contains(&self.area) {
            return Err(failed("draw the bar"));
        }
        self.events.borrow_mut().push(BarEvent::Drawn {
            area: self.area,
            content: content.clone(),
            background: config.colors.background.clone(),
        });
        Ok(())
    }

    fn hide(&mut self) -> Result<()> {
        self.events.borrow_mut().push(BarEvent::Hidden(self.area));
        Ok(())
    }
}

impl Drop for FakeBar {
    fn drop(&mut self) {
        self.events.borrow_mut().push(BarEvent::Dropped(self.area));
    }
}

/// The error a fake reports for a call set to fail.
fn failed(operation: &'static str) -> Error {
    Error::platform(operation, "the fake was set to fail")
}
