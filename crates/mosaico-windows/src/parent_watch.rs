//! [`ParentWatch`] on Windows: the parent's id from a process snapshot,
//! held open by a handle so its exit can be waited on.

use std::ops::ControlFlow;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::thread;

use mosaico_core::system::ParentWatch;
use windows::Win32::Foundation::{FILETIME, HANDLE};
use windows::Win32::System::Threading::{
    GetCurrentProcess, GetProcessTimes, INFINITE, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    PROCESS_SYNCHRONIZE, WaitForSingleObject,
};

use crate::process_list;

/// The process that started this one.
pub struct WindowsParentWatch {
    parent: Parent,
}

enum Parent {
    /// Held open, so its id is not reused while watched.
    Running { pid: u32, handle: OwnedHandle },
    /// It exited before the watch was made: its id names no process, or a
    /// later one that reused it.
    Exited,
    /// It could not be opened, so its exit cannot be heard.
    Unknown,
}

impl WindowsParentWatch {
    /// Watches the process that started this one.
    pub fn new() -> Self {
        let parent = match parent_pid() {
            None => Parent::Unknown,
            Some(pid) => open_parent(pid),
        };
        Self { parent }
    }
}

impl Default for WindowsParentWatch {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentWatch for WindowsParentWatch {
    fn pid(&self) -> Option<u32> {
        match &self.parent {
            Parent::Running { pid, .. } => Some(*pid),
            Parent::Exited | Parent::Unknown => None,
        }
    }

    fn wait(&self) {
        match &self.parent {
            Parent::Running { handle, .. } => {
                // SAFETY: the handle is open with SYNCHRONIZE for as long as
                // `self` lives, which outlasts the wait.
                unsafe { WaitForSingleObject(HANDLE(handle.as_raw_handle()), INFINITE) };
            }
            Parent::Exited => {}
            Parent::Unknown => loop {
                thread::park();
            },
        }
    }
}

/// The id of the process that started this one, from a snapshot of every
/// process. The id may since have been reused; `open_parent` checks.
fn parent_pid() -> Option<u32> {
    let me = std::process::id();
    process_list::walk(|entry| {
        if entry.th32ProcessID == me {
            ControlFlow::Break(entry.th32ParentProcessID)
        } else {
            ControlFlow::Continue(())
        }
    })
}

/// Opens process `pid`, unless it is not the parent: a process that
/// started after this one reused the id of a parent that exited.
fn open_parent(pid: u32) -> Parent {
    let access = PROCESS_SYNCHRONIZE | PROCESS_QUERY_LIMITED_INFORMATION;
    // SAFETY: OpenProcess returns a new handle that nothing else owns, so
    // the OwnedHandle closes it.
    let opened =
        unsafe { OpenProcess(access, false, pid).map(|h| OwnedHandle::from_raw_handle(h.0)) };
    let Ok(handle) = opened else {
        return Parent::Exited;
    };
    // SAFETY: the pseudo handle for the current process needs no closing.
    let me = unsafe { GetCurrentProcess() };
    match (started(HANDLE(handle.as_raw_handle())), started(me)) {
        (Some(parent), Some(me)) if parent > me => Parent::Exited,
        (Some(_), Some(_)) => Parent::Running { pid, handle },
        _ => Parent::Unknown,
    }
}

/// When the process `handle` names started, in 100 ns units.
fn started(handle: HANDLE) -> Option<u64> {
    let (mut created, mut exited) = (FILETIME::default(), FILETIME::default());
    let (mut kernel, mut user) = (FILETIME::default(), FILETIME::default());
    // SAFETY: the handle is open with query rights and every out pointer
    // is a live FILETIME.
    unsafe { GetProcessTimes(handle, &mut created, &mut exited, &mut kernel, &mut user) }.ok()?;
    Some((u64::from(created.dwHighDateTime) << 32) | u64::from(created.dwLowDateTime))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_running_parent_is_watched_by_its_id() {
        // Arrange, Act
        let watch = WindowsParentWatch::new();

        // Assert
        assert_eq!(watch.pid(), parent_pid());
        assert!(
            watch.pid().is_some(),
            "cargo started the test and still runs"
        );
    }

    #[test]
    fn a_process_started_later_is_not_taken_for_the_parent() {
        // Arrange
        let later = std::process::Command::new("cmd")
            .args(["/c", "ping -n 3 127.0.0.1 >nul"])
            .spawn()
            .unwrap();

        // Act
        let parent = open_parent(later.id());

        // Assert
        let mut later = later;
        let _ = later.kill();
        let _ = later.wait();
        assert!(matches!(parent, Parent::Exited));
    }
}
