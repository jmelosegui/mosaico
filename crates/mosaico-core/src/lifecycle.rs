//! Starting and stopping a daemon.
//!
//! What a daemon does around its loop is the same on every platform: it
//! clears what an update left behind, makes sure it is the only daemon
//! of its instance, records its process id, and undoes that once the
//! loop ends. The loop itself, which watches the desktop and answers the
//! CLI, is the platform's.

use std::path::PathBuf;

use crate::pid::PidFile;
use crate::system::SingleInstance;
use crate::update::Installer;
use crate::{Instance, Result};

/// What a daemon process starts and stops with.
pub struct Startup<'a> {
    /// The instance this process runs the daemon of. It must be the one
    /// the process resolved at startup, the one [`Instance::current`]
    /// returns, since the loop finds the config and the logs through
    /// [`Instance::current`] rather than through this. The platform's
    /// loop checks it with [`Instance::require_current`] before it
    /// starts.
    pub instance: &'a Instance,
    /// Keeps a second daemon of the instance from starting.
    pub lock: &'a dyn SingleInstance,
    /// Deletes what a self-update could not delete while the old
    /// executable ran.
    pub installer: &'a dyn Installer,
    /// The executable this process runs from, or `None` when it cannot
    /// be found, in which case nothing is deleted.
    pub exe: Option<PathBuf>,
    /// This process's id, which the PID file records.
    pub pid: u32,
    /// The platform's daemon loop. It runs until the daemon is told to
    /// stop, or can no longer be reached, and when it stops normally it
    /// puts every window back before it returns. When it cannot get going
    /// at all, it returns the error instead.
    pub serve: &'a dyn Fn(&Instance) -> Result<()>,
}

/// Runs a daemon from start to finish, and returns what its loop
/// returned.
///
/// In order, it:
///
/// 1. deletes what an update left beside the executable, ignoring a
///    failure: a leftover only takes disk space, and the next start tries
///    again;
/// 2. takes the instance's lock, and fails when another daemon holds it,
///    with [`Error::AlreadyRunning`](crate::Error::AlreadyRunning), or an
///    [`Error::Platform`](crate::Error::Platform) when this process may not
///    even open that daemon's lock, leaving that daemon's PID file alone;
/// 3. records the process id in the instance's PID file;
/// 4. logs that the daemon started, and runs the loop;
/// 5. logs the error the loop ended with, if any, removes the PID file,
///    and only then releases the lock, so a daemon that starts next never
///    has its PID file removed by this one.
///
/// Step 5 also runs, in that order, when a panic in the loop unwinds
/// through here. A build that aborts on a panic leaves the PID file
/// behind instead, as a crash does, and the CLI takes it for stale.
///
/// The daemon has no console, so it reports through the log, which the
/// loop sets up. A failure before the loop, such as another daemon
/// holding the lock, is only returned.
pub fn run(startup: Startup<'_>) -> Result<()> {
    if let Some(exe) = &startup.exe {
        let _ = startup.installer.remove_leftovers(exe);
    }
    let lock = startup.lock.acquire(startup.instance)?;
    let pid_file = PidFile::new(startup.instance.pid_path());
    pid_file.write(startup.pid)?;
    // Declared after the lock, so it is dropped first, also when a panic
    // unwinds.
    let pid_file = PidFileGuard(pid_file);
    crate::log_info!("Daemon started (PID: {})", startup.pid);

    let result = (startup.serve)(startup.instance);
    if let Err(e) = &result {
        crate::log_error!("Daemon stopped: {e}");
    }

    drop(pid_file);
    drop(lock);
    result
}

/// The daemon's PID file, removed when the guard is dropped.
struct PidFileGuard(PidFile);

impl Drop for PidFileGuard {
    fn drop(&mut self) {
        let _ = self.0.remove();
    }
}

#[cfg(test)]
#[path = "lifecycle_tests.rs"]
mod tests;
