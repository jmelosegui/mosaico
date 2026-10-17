//! Fakes of the bar's platform contracts.

use std::cell::RefCell;
use std::rc::Rc;

use crate::{Error, Result};

use super::{CpuTimes, LocalTime, NowPlaying, SystemInfo};

pub(super) use bar::NoPointer;
pub(super) use renderer::{BarEvent, FakeRenderer};

#[path = "fake_bar.rs"]
mod bar;
#[path = "fake_layout.rs"]
pub(in crate::bar) mod layout;
#[path = "fake_renderer.rs"]
mod renderer;

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

/// The error a fake reports for a call set to fail.
pub(in crate::bar) fn failed(operation: &'static str) -> Error {
    Error::platform(operation, "the fake was set to fail")
}
