//! What mosaico asks of the OS besides talking to the daemon: setting up
//! each process, starting and stopping processes, keeping a second daemon
//! from starting, and registering for start on login.

mod autostart;
mod autostart_status;
mod file_probe;
mod setup;
mod single_instance;

use crate::Result;

pub use autostart::{Autostart, LaunchCommand};
pub use autostart_status::AutostartStatus;
pub use file_probe::FileProbe;
pub use setup::ProcessSetup;
pub use single_instance::{InstanceLock, SingleInstance};

/// Starting and stopping daemon processes.
///
/// # Contract for platform implementations
///
/// - `launch_daemon` starts a daemon of the current instance in the
///   background and returns its process id without waiting for it. The
///   daemon must not be tied to the caller's console: closing the
///   terminal or pressing Ctrl+C there must not stop it, and the caller
///   must not wait on handles the daemon inherited. It inherits the
///   caller's environment, so it belongs to the same instance. It fails
///   with [`Error::Platform`](crate::Error::Platform) when the process
///   cannot be started.
/// - `is_alive` answers for any process id, not only daemons; an id that
///   was never used, or whose process has exited, is not alive.
/// - `kill` ends the process without giving it a chance to clean up. It
///   fails with [`Error::Platform`](crate::Error::Platform) when the
///   process could not be ended, for example because the caller is not
///   allowed to.
/// - `kill` also fails for a process that has already exited, even one
///   that was alive at the last `is_alive`.
pub trait ProcessControl {
    /// Starts a daemon in the background and returns its process id.
    fn launch_daemon(&self) -> Result<u32>;

    /// Whether a process with this id is running.
    fn is_alive(&self, pid: u32) -> bool;

    /// Ends a process forcefully.
    fn kill(&self, pid: u32) -> Result<()>;
}
