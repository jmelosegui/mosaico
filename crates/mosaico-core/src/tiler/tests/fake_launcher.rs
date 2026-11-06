//! A launcher that records what it was asked to start, for the tests of
//! launch bindings.

use std::cell::RefCell;

use crate::system::Launcher;
use crate::{Error, Result};

/// Records every program it is asked to start, with its arguments, and
/// fails for the commands it is told it cannot find.
#[derive(Default)]
pub(in crate::tiler) struct RecordingLauncher {
    started: RefCell<Vec<(String, Vec<String>)>>,
    missing: Vec<String>,
}

impl RecordingLauncher {
    /// A launcher that cannot find `command`.
    pub(in crate::tiler) fn without(command: &str) -> Self {
        Self {
            missing: vec![command.into()],
            ..Self::default()
        }
    }

    /// The programs started so far, oldest first.
    pub(in crate::tiler) fn started(&self) -> Vec<(String, Vec<String>)> {
        self.started.borrow().clone()
    }
}

impl Launcher for RecordingLauncher {
    fn launch(&self, command: &str, args: &[String]) -> Result<()> {
        if self.missing.iter().any(|missing| missing == command) {
            return Err(Error::platform("start an application", "not found"));
        }
        self.started
            .borrow_mut()
            .push((command.to_string(), args.to_vec()));
        Ok(())
    }
}
