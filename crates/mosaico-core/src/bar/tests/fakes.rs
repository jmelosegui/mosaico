//! Fakes of the bar's platform contracts.

use std::cell::RefCell;
use std::rc::Rc;

use crate::{Error, Result};

use super::{CpuTimes, LocalTime, NowPlaying, SystemInfo};

/// What a [`FakeSystem`] reports.
#[derive(Debug, Default)]
pub(super) struct Machine {
    /// Processor counters. `None` makes reading them fail.
    pub(super) cpu: Option<CpuTimes>,
    /// Memory in use. `None` makes reading it fail.
    pub(super) memory: Option<u32>,
    /// What is playing.
    pub(super) playing: Option<NowPlaying>,
    /// The local time.
    pub(super) time: LocalTime,
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
}

impl SystemInfo for FakeSystem {
    fn cpu_times(&self) -> Result<CpuTimes> {
        self.0
            .borrow()
            .cpu
            .ok_or_else(|| failed("read the processor times"))
    }

    fn memory_usage(&self) -> Result<u32> {
        self.0
            .borrow()
            .memory
            .ok_or_else(|| failed("read the memory"))
    }

    fn now_playing(&self) -> Result<Option<NowPlaying>> {
        Ok(self.0.borrow().playing.clone())
    }

    fn local_time(&self) -> LocalTime {
        self.0.borrow().time
    }
}

/// The error a fake reports for a reading set to fail.
fn failed(operation: &'static str) -> Error {
    Error::platform(operation, "the fake was set to fail")
}
